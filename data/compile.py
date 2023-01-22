import logging
import os

from processors import (
    areatree,
    coords,
    export,
    images,
    maps,
    merge,
    nat,
    roomfinder,
    search,
    sections,
    sitemap,
    structure,
    tumonline,
)
from utils import setup_logging

DEBUG_MODE = "GIT_COMMIT_SHA" not in os.environ


# pylint: disable=too-many-locals,too-many-statements
def main():
    """Main function"""
    # --- Read base data ---
    logging.info("-- 00 areatree")
    data = areatree.read_areatree()

    logging.info("-- 01 areas extendend")
    data = merge.patch_areas(data, "sources/01_areas-extended.yaml")

    logging.info("-- 02 rooms extendend")
    data = merge.patch_rooms(data, "sources/02_rooms-extended.yaml")
    merge.add_coordinates(data, "sources/coordinates/")

    # Add source information for these entries, which are up to here
    # always declared by navigatum
    for _id, entry in data.items():
        entry.setdefault("sources", {"base": [{"name": "NavigaTUM"}]})

    # --- Buildings ---
    logging.info("-- 10 Roomfinder buildings")
    roomfinder.merge_roomfinder_buildings(data)

    logging.info("-- 11 TUMonline buildings")
    tumonline.merge_tumonline_buildings(data)

    logging.info("-- 12 NAT buildings")
    nat.merge_nat_buildings(data)

    # --- Rooms ---
    # TUMonline is used as base
    logging.info("-- 15 TUMonline rooms")
    tumonline.merge_tumonline_rooms(data)

    # merge data which is contributed by the mytum roomfinder (mostly coordinates)
    logging.info("-- 16 Roomfinder rooms")
    roomfinder.merge_roomfinder_rooms(data)

    # merge data which is contributed by the nat roomfinder (additonal rooms, seating information, ...)
    logging.info("-- 17 NAT rooms")
    nat.merge_nat_rooms(data)

    # At this point, no more areas or rooms will be added or removed.
    # --- Make data more coherent ---
    logging.info("-- 30 Add children properties")
    structure.add_children_properties(data)

    logging.info("-- 33 Add (structural) stats")
    structure.add_stats(data)

    logging.info("-- 34 Infer more props")
    structure.infer_addresses(data)

    # TODO: Does it make sense to introduce a type 'sub_building' here?

    logging.info("-- 35 Infer the common_name for every type")
    structure.infer_type_common_name(data)

    logging.info("-- 40 Coordinates")
    coords.add_and_check_coords(data)

    logging.info("-- 45 Roomfinder maps")
    maps.roomfinder_maps(data)

    logging.info("-- 46 Overlay maps")
    maps.add_overlay_maps(data)

    logging.info("-- 50 Convert, resize and crop the images for different resolutions and formats")
    images.resize_and_crop()
    logging.info("-- 51 Add image information")
    images.add_img(data)

    logging.info("-- 80 Generate info card")
    sections.extract_tumonline_props(data)
    sections.compute_floor_prop(data)
    sections.compute_props(data)
    sections.localize_links(data)

    logging.info("-- 81 Generate overview sections")
    sections.generate_buildings_overview(data)
    sections.generate_rooms_overview(data)

    logging.info("-- 90 Search: Build base ranking")
    search.add_ranking_base(data)

    logging.info("-- 97 Search: Get combined ranking")
    search.add_ranking_combined(data)

    logging.info("-- 99 Search: Export")
    export.export_for_search(data, "output/search_data.json")

    logging.info("-- 100 Export: API")
    export.export_for_api(data, "output/api_data.json")

    # Sitemap is only generated for deployments
    logging.info("-- 101 Extra: Sitemap")
    sitemap.generate_sitemap()


if __name__ == "__main__":
    setup_logging(level=logging.DEBUG if DEBUG_MODE else logging.INFO)

    # Pillow prints all imported modules to the debug stream
    logging.getLogger("PIL").setLevel(logging.INFO)
    main()
