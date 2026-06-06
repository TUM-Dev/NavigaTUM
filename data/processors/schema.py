import dataframely as dy
import polars as pl


class LocationSchema(dy.Schema):
    """Schema for the flat wide table representing all location entries."""

    # --- Core ---
    id = dy.String(nullable=False)
    type = dy.String(nullable=False)
    name = dy.String(nullable=False)
    name_de = dy.String()
    name_en = dy.String()
    short_name = dy.String()
    short_name_de = dy.String()
    short_name_en = dy.String()
    visible_id = dy.String()

    # --- Hierarchy ---
    parents = dy.List(inner=dy.String())
    b_prefix = dy.String()
    b_prefix_list = dy.List(inner=dy.String())

    # --- Coords ---
    coords_lat = dy.Float64()
    coords_lon = dy.Float64()
    coords_source = dy.String()
    coords_accuracy = dy.String()
    coords_utm_easting = dy.Float64()
    coords_utm_northing = dy.Float64()
    coords_utm_zone_number = dy.Int64()
    coords_utm_zone_letter = dy.String()

    # --- Props: IDs ---
    props_ids_b_id = dy.String()
    props_ids_roomcode = dy.String()
    props_ids_arch_name = dy.String()

    # --- Props: Address ---
    props_address_street = dy.String()
    props_address_plz_place = dy.String()
    props_address_source = dy.String()

    # --- Props: Stats ---
    props_stats_n_rooms = dy.Int64()
    props_stats_n_rooms_reg = dy.Int64()
    props_stats_n_buildings = dy.Int64()
    props_stats_n_seats = dy.Int64()
    props_stats_n_seats_sitting = dy.Int64()
    props_stats_n_seats_standing = dy.Int64()
    props_stats_n_seats_wheelchair = dy.Int64()

    # --- Props: Operator ---
    props_operator_code = dy.String()
    props_operator_name_de = dy.String()
    props_operator_name_en = dy.String()
    props_operator_url = dy.String()
    props_operator_id = dy.Int64()

    # --- Props: Complex (JSON-serialized) ---
    props_calendar_url = dy.String()
    props_tumonline_room_nr = dy.Int64()
    props_floors_json = dy.String()
    props_computed_json = dy.String()
    props_links_json = dy.String()
    props_generic_json = dy.String()
    props_comment_de = dy.String()
    props_comment_en = dy.String()

    # --- Usage ---
    usage_name = dy.String()
    usage_name_de = dy.String()
    usage_name_en = dy.String()
    usage_din_277 = dy.String()
    usage_din_277_desc = dy.String()

    # --- Ranking ---
    ranking_rank_type = dy.Int64()
    ranking_rank_usage = dy.Int64()
    ranking_rank_boost = dy.Int64()
    ranking_rank_custom = dy.Int64()
    ranking_rank_combined = dy.Int64()

    # --- External data (JSON-serialized, internal pipeline use) ---
    tumonline_data_json = dy.String()
    roomfinder_data_json = dy.String()

    # --- Late-stage ---
    arch_name = dy.String()
    aliases_json = dy.String()
    imgs_json = dy.String()
    type_common_name = dy.String()
    type_common_name_de = dy.String()
    type_common_name_en = dy.String()

    # --- Sections (JSON-serialized) ---
    sections_buildings_overview_json = dy.String()
    sections_rooms_overview_json = dy.String()

    # --- Metadata ---
    sources_base_json = dy.String()
    sources_patched = dy.Bool()
    data_quality_json = dy.String()
    generators_json = dy.String()

    # --- Structural ---
    children = dy.List(inner=dy.String())
    children_flat = dy.List(inner=dy.String())

    # --- Maps ---
    maps_default = dy.String()

    # --- Description ---
    description_json = dy.String()
    external_data_json = dy.String()

    # --- AStA Iris learning-room coverage ---
    has_iris_coverage = dy.Bool()

    @dy.rule()
    def type_is_valid(cls) -> pl.Expr:
        """Validate that the type column only contains the allowed entry kinds."""
        valid_types = ["root", "site", "campus", "area", "joined_building", "building", "room", "virtual_room", "poi"]
        return pl.col("type").is_in(valid_types)
