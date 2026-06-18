import csv
import logging
from concurrent.futures import Future, ThreadPoolExecutor
from datetime import datetime
from multiprocessing import Process

import polars as pl
import processors.areatree.process as areatree
import yaml
from external.loaders.opening_hours import load_opening_hours
from pipeline_types import Entry
from processors import (
    aliases,
    coords,
    export,
    images,
    iris,
    merge,
    opening_hours,
    poi,
    roomfinder,
    search,
    sections,
    sitemap,
    structure,
    studierendenwerk,
    tumonline,
)
from processors import (
    ub_tum as ub_tum_processor,
)
from processors.df_utils import ensure_columns
from processors.exports import location_images as location_images_export
from processors.exports import operators as operators_export
from processors.exports import parents as parents_export
from processors.exports import ranking_factors as ranking_factors_export
from processors.exports import sources as sources_export
from processors.exports import urls as urls_export
from processors.exports import usages as usages_export
from processors.sitemap import SimplifiedSitemaps
from utils import DEV_MODE, setup_logging

_logger = logging.getLogger(__name__)

# All columns that may be referenced by downstream processors.
# Ensures they exist (as null) before any processor tries to read them.
_ALL_NULLABLE_COLUMNS: dict[str, pl.DataType] = {
    "sources_base_json": pl.Utf8(),
    "sources_patched": pl.Boolean(),
    "roomfinder_data_json": pl.Utf8(),
    "tumonline_data_json": pl.Utf8(),
    "coords_lat": pl.Float64(),
    "coords_lon": pl.Float64(),
    "coords_source": pl.Utf8(),
    "coords_accuracy": pl.Utf8(),
    "coords_utm_easting": pl.Float64(),
    "coords_utm_northing": pl.Float64(),
    "coords_utm_zone_number": pl.Int64(),
    "coords_utm_zone_letter": pl.Utf8(),
    "props_ids_b_id": pl.Utf8(),
    "props_ids_roomcode": pl.Utf8(),
    "props_ids_arch_name": pl.Utf8(),
    "props_address_street": pl.Utf8(),
    "props_address_plz_place": pl.Utf8(),
    "props_address_source": pl.Utf8(),
    "props_stats_n_rooms": pl.Int64(),
    "props_stats_n_rooms_reg": pl.Int64(),
    "props_stats_n_buildings": pl.Int64(),
    "props_stats_n_seats": pl.Int64(),
    "props_stats_n_seats_sitting": pl.Int64(),
    "props_stats_n_seats_standing": pl.Int64(),
    "props_stats_n_seats_wheelchair": pl.Int64(),
    "props_operator_code": pl.Utf8(),
    "props_operator_name_de": pl.Utf8(),
    "props_operator_name_en": pl.Utf8(),
    "props_operator_url": pl.Utf8(),
    "props_operator_id": pl.Int64(),
    "props_calendar_url": pl.Utf8(),
    "props_tumonline_room_nr": pl.Int64(),
    "props_floors_json": pl.Utf8(),
    "props_computed_json": pl.Utf8(),
    "props_links_json": pl.Utf8(),
    "props_generic_json": pl.Utf8(),
    "props_comment_de": pl.Utf8(),
    "props_comment_en": pl.Utf8(),
    "usage_name_de": pl.Utf8(),
    "usage_name_en": pl.Utf8(),
    "usage_din_277": pl.Utf8(),
    "usage_din_277_desc": pl.Utf8(),
    "ranking_rank_type": pl.Int64(),
    "ranking_rank_usage": pl.Int64(),
    "ranking_rank_boost": pl.Int64(),
    "ranking_rank_custom": pl.Int64(),
    "ranking_rank_combined": pl.Int64(),
    "arch_name": pl.Utf8(),
    "aliases": pl.List(pl.Utf8()),
    "imgs_json": pl.Utf8(),
    "type_common_name": pl.Utf8(),
    "type_common_name_de": pl.Utf8(),
    "type_common_name_en": pl.Utf8(),
    "sections_buildings_overview_json": pl.Utf8(),
    "sections_rooms_overview_json": pl.Utf8(),
    "data_quality_json": pl.Utf8(),
    "generators_json": pl.Utf8(),
    "description_json": pl.Utf8(),
    "maps_default": pl.Utf8(),
    "external_data_json": pl.Utf8(),
    "generate_rooms_overview_json": pl.Utf8(),
    "children": pl.List(pl.Utf8()),
    "children_flat": pl.List(pl.Utf8()),
}


def main() -> None:
    """Process data and generate output."""
    _logger.info("-- (Parallel) Convert, resize and crop the images for different resolutions and formats")
    resizer = Process(target=images.resize_and_crop)
    resizer.start()

    # Kick off sitemap network IO in worker threads so it overlaps with the main pipeline.
    # The threads release the GIL during socket reads, so the merge/lazy work runs unimpeded.
    # try/finally guarantees the executor is shut down even on early failure.
    sitemap_io = ThreadPoolExecutor(max_workers=3, thread_name_prefix="sitemap-io")
    try:
        fut_old_data = sitemap_io.submit(sitemap.fetch_old_data)
        fut_old_sitemaps = sitemap_io.submit(sitemap.fetch_online_sitemaps)
        fut_web_sitemap = sitemap_io.submit(sitemap.download_online_sitemap, sitemap.WEB_SITEMAP_URL)

        _run_pipeline(
            resizer=resizer,
            fut_old_data=fut_old_data,
            fut_old_sitemaps=fut_old_sitemaps,
            fut_web_sitemap=fut_web_sitemap,
        )
    finally:
        sitemap_io.shutdown(wait=True)


def _run_pipeline(
    *,
    resizer: Process,
    fut_old_data: Future[list[Entry]],
    fut_old_sitemaps: Future[SimplifiedSitemaps],
    fut_web_sitemap: Future[dict[str, datetime]],
) -> None:
    # --- Read base data ---
    _logger.info("-- 00 areatree")
    df = areatree.read_areatree()

    _logger.info("-- 01 areas extended")
    df = merge.patch_areas(df)

    # --- Decomposed CSV overrides that create/patch entries ---
    # Applied early (before source annotation and merges) so entries exist
    # for TUMonline/Roomfinder matching, matching original patch_rooms behavior
    _logger.info("-- 02 room overrides (names, usages, ranking)")
    df = merge.add_names(df)
    df = merge.add_usages(df)
    df = merge.add_ranking(df)

    # Ensure all nullable columns exist before downstream processors reference them
    df = ensure_columns(df, _ALL_NULLABLE_COLUMNS)

    # Add source information for entries declared by NavigaTUM
    df = df.with_columns(pl.col("sources_base_json").fill_null('[{"name":"NavigaTUM"}]'))

    # --- Buildings (eager: Python dict lookups) ---
    _logger.info("-- 10 Roomfinder buildings")
    df = roomfinder.merge_roomfinder_buildings(df)

    _logger.info("-- 11 TUMonline buildings")
    df = tumonline.merge_tumonline_buildings(df)

    # --- Rooms (eager: Python dict lookups, concat) ---
    _logger.info("-- 15 TUMonline rooms")
    df = tumonline.merge_tumonline_rooms(df)

    _logger.info("-- 16 Roomfinder rooms")
    df = roomfinder.merge_roomfinder_rooms(df)

    # --- POIs (eager: Python dict lookups, concat) ---
    _logger.info("-- 21 POIs")
    df = poi.merge_poi(df)

    # Re-ensure columns after concat (new rows from rooms/POIs may lack some columns)
    df = ensure_columns(df, _ALL_NULLABLE_COLUMNS)

    # --- Decomposed CSV/YAML overrides (metadata only, entries already created at step 02) ---
    _logger.info("-- 22 Decomposed overrides (comments, links, opening hours)")
    df = merge.add_comments(df)
    df = merge.add_links(df)
    # Hand-authored schedules win over scraped mensa/library hours on an id collision.
    schedules = pl.concat(
        [load_opening_hours(), studierendenwerk.mensa_opening_hours(), ub_tum_processor.ub_tum_opening_hours()],
        how="vertical",
    ).unique(subset="id", keep="first", maintain_order=True)
    # Expands lecture:/break: macros; fails the build on an unknown id or a schedule
    # that does not reduce to plain OSM.
    df = opening_hours.merge_opening_hours(df, schedules=schedules)

    # Stamp the eat-api canteen slug so the detail page can fetch the live menu client-side;
    # no menu payload is baked into the build.
    df = studierendenwerk.stamp_canteen_ids(df)

    # Entries that only appear in comments/links CSVs (not in names.csv) were not created
    # at step 02, so they don't have NavigaTUM source. Prepend it now.
    comment_ids: set[str] = set()
    if merge.COMMENTS_CSV.exists():
        with merge.COMMENTS_CSV.open() as f:
            comment_ids.update(row["id"] for row in csv.DictReader(f))
    if merge.LINKS_YAML.exists():
        with merge.LINKS_YAML.open() as f:
            links_data = yaml.safe_load(f) or {}
        comment_ids.update(str(k) for k in links_data)
    if comment_ids:
        df = df.with_columns(
            pl.when(
                pl.col("id").is_in(list(comment_ids))
                & pl.col("sources_base_json").is_not_null()
                & ~pl.col("sources_base_json").str.contains("NavigaTUM")
            )
            .then(pl.lit('[{"name":"NavigaTUM"},') + pl.col("sources_base_json").str.strip_prefix("["))
            .otherwise(pl.col("sources_base_json"))
            .alias("sources_base_json")
        )

    # --- Structure: lazy block for children + type_common_name,
    #     eager for stats + addresses (need .height / logging) ---
    _logger.info("-- 30 Add children properties")
    lf = df.lazy()
    lf = structure.add_children_properties(lf)
    df = lf.collect()

    _logger.info("-- 33 Add (structural) stats")
    df = structure.add_stats(df)

    _logger.info("-- 34 Infer more props")
    df = structure.infer_addresses(df)

    _logger.info("-- 35 Infer the common_name for every type")
    lf = df.lazy()
    lf = structure.infer_type_common_name(lf)
    df = lf.collect()

    # --- Coordinates (eager: map_elements, error checks) ---
    _logger.info("-- 40 Coordinates")
    df = merge.add_coordinates(df)
    df = coords.add_and_check_coords(df)

    # --- Images (eager: file IO, set lookup) ---
    _logger.info("-- 51 Add image information")
    df = images.add_img(df)

    # --- Sections: lazy for extract_tumonline_props, eager for the rest ---
    _logger.info("-- 80 Generate info card")
    lf = df.lazy()
    lf = sections.extract_tumonline_props(lf)
    df = lf.collect()
    df = sections.compute_floor_prop(df)
    df = poi.propagate_poi_floors(df)
    df = sections.compute_props(df)
    df = sections.localize_links(df)

    _logger.info("-- 81 Generate overview sections")
    df = sections.generate_buildings_overview(df)
    df = sections.generate_rooms_overview(df)

    # --- Search + Aliases: lazy block (pure expressions) ---
    _logger.info("-- 90 Search: Build base ranking")
    lf = df.lazy()
    lf = search.add_ranking_base(lf)

    _logger.info("-- 97 Search: Get combined ranking")
    lf = search.add_ranking_combined(lf)

    _logger.info("-- 98 Aliases: extract aliases")
    lf = aliases.add_aliases(lf, aliases.building_short_name_lookup(df))

    # Filter root and collect for export
    lf = lf.filter(pl.col("id") != "root")
    df = lf.collect()

    # Needs arch_name (added at step 98) to join the Iris roster against NavigaTUM aliases.
    _logger.info("-- 99 Studentische Vertretung IRIS learning-room coverage")
    df = iris.add_iris_coverage(df)

    _logger.info("-- 100 Export and generate Sitemap")
    data = export.reconstruct_data(df)
    export.export_for_search(data)
    export.export_for_api(data)
    export.export_for_status()
    export.export_known_usages(df)
    export.export_tumonline_orgs_parquet()
    export.export_known_orgs()
    export.export_events_parquet()
    ranking_factors_export.export_ranking_factors_parquet(df)
    operators_export.export_operators_de_parquet(df)
    operators_export.export_operators_en_parquet(df)
    sources_export.export_sources_parquet(df)
    usages_export.export_usages_parquet(df)
    urls_export.export_urls_de_parquet(df)
    urls_export.export_urls_en_parquet(df)
    parents_export.export_parents_parquet(df)
    location_images_export.export_location_images_parquet(df)
    sitemap.generate_sitemap(
        old_data=fut_old_data.result(),
        old_sitemaps=fut_old_sitemaps.result(),
        web_sitemap=fut_web_sitemap.result(),
    )

    resizer.join(timeout=60 * 4)
    if resizer.exitcode != 0:
        raise RuntimeError("Resizer process during the execution of the script")


if __name__ == "__main__":
    setup_logging(level=logging.DEBUG if DEV_MODE else logging.INFO)
    logging.getLogger("PIL").setLevel(logging.INFO)
    main()
