import dataframely as dy
import polars as pl


class BuildingsSchema(dy.Schema):
    """Schema for the Roomfinder building catalogue (`buildings_roomfinder.csv`)."""

    b_alias = dy.String(nullable=False)
    b_area = dy.String(nullable=False)
    b_id = dy.String(nullable=False)
    b_name = dy.String(nullable=False)
    b_room_count = dy.Int64(nullable=False)
    lat = dy.Float64(nullable=False)
    lon = dy.Float64(nullable=False)
    default_map_scale = dy.String(nullable=True)
    default_map_id = dy.String(nullable=True)
    default_map_name = dy.String(nullable=True)
    default_map_width = dy.Int64(nullable=True)
    default_map_height = dy.Int64(nullable=True)
    maps_json = dy.String(nullable=False)


class RoomsSchema(dy.Schema):
    """Schema for the Roomfinder room catalogue (`rooms_roomfinder.csv`)."""

    lat = dy.Float64(nullable=False)
    lon = dy.Float64(nullable=False)
    r_alias = dy.String(nullable=False)
    r_id = dy.String(nullable=False)
    r_level = dy.String(nullable=False)
    r_number = dy.String(nullable=False)
    b_alias = dy.String(nullable=False)
    b_area = dy.String(nullable=False)
    b_id = dy.String(nullable=False)
    b_name = dy.String(nullable=False)
    default_map_scale = dy.String(nullable=False)
    default_map_id = dy.String(nullable=False)
    default_map_name = dy.String(nullable=False)
    default_map_width = dy.Int64(nullable=False)
    default_map_height = dy.Int64(nullable=False)
    maps_json = dy.String(nullable=False)
    metas_json = dy.String(nullable=False)


class MapsSchema(dy.Schema):
    """Schema for the Roomfinder map catalogue (`maps_roomfinder.csv`)."""

    id = dy.String(nullable=False)
    desc = dy.String(nullable=False)
    height = dy.Int64(nullable=False)
    width = dy.Int64(nullable=False)
    scale = dy.String(nullable=False)
    latlonbox_north = dy.Float64(nullable=False)
    latlonbox_south = dy.Float64(nullable=False)
    latlonbox_east = dy.Float64(nullable=False)
    latlonbox_west = dy.Float64(nullable=False)
    latlonbox_rotation = dy.Float64(nullable=False)
    file = dy.String(nullable=False)
    source = dy.String(nullable=False)
