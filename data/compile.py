import logging
from multiprocessing import Process

import polars as pl

import processors.areatree.process as areatree
from processors import (
    aliases,
    coords,
    export,
    images,
    merge,
    poi,
    roomfinder,
    search,
    sections,
    sitemap,
    structure,
    tumonline,
)
from processors.df_utils import ensure_columns
from utils import DEV_MODE, setup_logging

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
    "aliases_json": pl.Utf8(),
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
    logging.info("-- (Parallel) Convert, resize and crop the images for different resolutions and formats")
    resizer = Process(target=images.resize_and_crop)
    resizer.start()

    # --- Read base data ---
    logging.info("-- 00 areatree")
    df = areatree.read_areatree()

    logging.info("-- 01 areas extended")
    df = merge.patch_areas(df)

    # --- Decomposed CSV overrides that create/patch entries ---
    # Applied early (before source annotation and merges) so entries exist
    # for TUMonline/Roomfinder matching, matching original patch_rooms behavior
    logging.info("-- 02 room overrides (names, usages, ranking)")
    df = merge.add_names(df)
    df = merge.add_usages(df)
    df = merge.add_ranking(df)

    # Ensure all nullable columns exist before downstream processors reference them
    df = ensure_columns(df, _ALL_NULLABLE_COLUMNS)

    # Add source information for entries declared by NavigaTUM
    df = df.with_columns(pl.col("sources_base_json").fill_null('[{"name":"NavigaTUM"}]'))

    # --- Buildings (eager: Python dict lookups) ---
    logging.info("-- 10 Roomfinder buildings")
    df = roomfinder.merge_roomfinder_buildings(df)

    logging.info("-- 11 TUMonline buildings")
    df = tumonline.merge_tumonline_buildings(df)

    # --- Rooms (eager: Python dict lookups, concat) ---
    logging.info("-- 15 TUMonline rooms")
    df = tumonline.merge_tumonline_rooms(df)

    logging.info("-- 16 Roomfinder rooms")
    df = roomfinder.merge_roomfinder_rooms(df)

    # --- POIs (eager: Python dict lookups, concat) ---
    logging.info("-- 21 POIs")
    df = poi.merge_poi(df)

    # Re-ensure columns after concat (new rows from rooms/POIs may lack some columns)
    df = ensure_columns(df, _ALL_NULLABLE_COLUMNS)

    # --- Decomposed CSV/YAML overrides (metadata only, entries already created at step 02) ---
    logging.info("-- 22 Decomposed overrides (comments, links)")
    df = merge.add_comments(df)
    df = merge.add_links(df)

    # Entries that only appear in comments/links CSVs (not in names.csv) were not created
    # at step 02, so they don't have NavigaTUM source. Prepend it now.
    comment_ids = set()
    if merge.COMMENTS_CSV.exists():
        import csv as csv_mod

        with merge.COMMENTS_CSV.open() as f:
            comment_ids.update(row["id"] for row in csv_mod.DictReader(f))
    if merge.LINKS_YAML.exists():
        import yaml as yaml_mod

        with merge.LINKS_YAML.open() as f:
            links_data = yaml_mod.safe_load(f) or {}
        comment_ids.update(str(k) for k in links_data)
    if comment_ids:
        ids_series = pl.Series("_cid", list(comment_ids))
        df = df.with_columns(
            pl.when(
                pl.col("id").is_in(ids_series)
                & pl.col("sources_base_json").is_not_null()
                & ~pl.col("sources_base_json").str.contains("NavigaTUM")
            )
            .then(pl.lit('[{"name":"NavigaTUM"},') + pl.col("sources_base_json").str.strip_prefix("["))
            .otherwise(pl.col("sources_base_json"))
            .alias("sources_base_json")
        )

    # --- Structure: lazy block for children + type_common_name,
    #     eager for stats + addresses (need .height / logging) ---
    logging.info("-- 30 Add children properties")
    lf = df.lazy()
    lf = structure.add_children_properties(lf)
    df = lf.collect()

    logging.info("-- 33 Add (structural) stats")
    df = structure.add_stats(df)

    logging.info("-- 34 Infer more props")
    df = structure.infer_addresses(df)

    logging.info("-- 35 Infer the common_name for every type")
    lf = df.lazy()
    lf = structure.infer_type_common_name(lf)
    df = lf.collect()

    # --- Coordinates (eager: map_elements, error checks) ---
    logging.info("-- 40 Coordinates")
    df = merge.add_coordinates(df)
    df = coords.add_and_check_coords(df)

    # --- Images (eager: file IO, set lookup) ---
    logging.info("-- 51 Add image information")
    df = images.add_img(df)

    # --- Sections: lazy for extract_tumonline_props, eager for the rest ---
    logging.info("-- 80 Generate info card")
    lf = df.lazy()
    lf = sections.extract_tumonline_props(lf)
    df = lf.collect()
    df = sections.compute_floor_prop(df)
    df = sections.compute_props(df)
    df = sections.localize_links(df)

    logging.info("-- 81 Generate overview sections")
    df = sections.generate_buildings_overview(df)
    df = sections.generate_rooms_overview(df)

    # --- Search + Aliases: lazy block (pure expressions) ---
    logging.info("-- 90 Search: Build base ranking")
    lf = df.lazy()
    lf = search.add_ranking_base(lf)

    logging.info("-- 97 Search: Get combined ranking")
    lf = search.add_ranking_combined(lf)

    logging.info("-- 98 Aliases: extract aliases")
    lf = aliases.add_aliases(lf)

    # Filter root and collect for export
    lf = lf.filter(pl.col("id") != "root")
    df = lf.collect()

    logging.info("-- 100 Export and generate Sitemap")
    data = export.reconstruct_data(df)
    export.export_for_search(data)
    export.export_for_api(data)
    export.export_for_status()
    sitemap.generate_sitemap()

    resizer.join(timeout=60 * 4)
    if resizer.exitcode != 0:
        raise RuntimeError("Resizer process during the execution of the script")


if __name__ == "__main__":
    setup_logging(level=logging.DEBUG if DEV_MODE else logging.INFO)
    logging.getLogger("PIL").setLevel(logging.INFO)
    main()
