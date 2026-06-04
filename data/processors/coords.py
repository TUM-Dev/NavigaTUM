import logging

import polars as pl
import utm
from utils import distance_via_great_circle

from processors.df_utils import ensure_columns

_logger = logging.getLogger(__name__)

MAX_DISTANCE_METERS_FROM_PARENT = 400


def assert_buildings_have_coords(df: pl.DataFrame) -> None:
    """
    Assert that all buildings have coordinates.

    The inference of coordinates in further functions for all entries is based on the
    coordinates of buildings, so it is necessary, that at least all buildings have
    a coordinate.
    """
    buildings_without = df.filter((pl.col("type") == "building") & pl.col("coords_lat").is_null())
    if buildings_without.height > 0:
        ids = buildings_without["id"].to_list()
        names = buildings_without["name"].to_list()
        msg = "\n".join(f"{i}: {n}" for i, n in zip(ids, names, strict=True))
        raise RuntimeError(f"No coordinates known for the following buildings:\n{msg}")


def _convert_utm_to_latlon(df: pl.DataFrame) -> pl.DataFrame:
    """Convert UTM coordinates to lat/lon where lat is missing but UTM is present."""
    needs_conversion = df.filter(pl.col("coords_lat").is_null() & pl.col("coords_utm_easting").is_not_null())
    if needs_conversion.height == 0:
        return df

    converted = (
        needs_conversion.with_columns(
            pl.struct("coords_utm_easting", "coords_utm_northing", "coords_utm_zone_number", "coords_utm_zone_letter")
            .map_elements(
                lambda s: utm.to_latlon(
                    s["coords_utm_easting"],
                    s["coords_utm_northing"],
                    int(s["coords_utm_zone_number"]),
                    s["coords_utm_zone_letter"],
                ),
                return_dtype=pl.List(pl.Float64),
            )
            .alias("_latlon")
        )
        .with_columns(
            pl.col("_latlon").list.get(0).alias("coords_lat"),
            pl.col("_latlon").list.get(1).alias("coords_lon"),
        )
        .drop("_latlon")
    )

    # Update the original dataframe: replace the rows that were converted
    df_rest = df.filter(~(pl.col("coords_lat").is_null() & pl.col("coords_utm_easting").is_not_null()))
    return pl.concat([df_rest, converted], how="diagonal_relaxed").sort("id")


def assign_coordinates(df: pl.DataFrame) -> pl.DataFrame:
    """Assign coordinates to all entries (except root) and make sure they match the data format."""
    error = False

    # Ensure coordinate columns exist
    df = ensure_columns(
        df,
        {
            "coords_lat": pl.Float64(),
            "coords_lon": pl.Float64(),
            "coords_source": pl.Utf8(),
            "coords_accuracy": pl.Utf8(),
            "coords_utm_easting": pl.Float64(),
            "coords_utm_northing": pl.Float64(),
            "coords_utm_zone_number": pl.Int64(),
            "coords_utm_zone_letter": pl.Utf8(),
        },
    )

    # 1. Convert UTM to lat/lon where needed
    df = _convert_utm_to_latlon(df)

    # 2. Set source to "navigatum" where coords exist but no source is set
    df = df.with_columns(
        pl.when(pl.col("coords_lat").is_not_null() & pl.col("coords_source").is_null())
        .then(pl.lit("navigatum"))
        .otherwise(pl.col("coords_source"))
        .alias("coords_source")
    )

    # 3a. POIs whose direct parent is a room: inherit the room's coords first,
    # so the marker lands at the room centroid instead of the building centroid.
    # accuracy stays "building" — the existing "inaccurate position" toast
    # (DetailsContentSidebar.vue) keeps encouraging the user to refine the coord.
    room_coords = df.filter((pl.col("type") == "room") & pl.col("coords_lat").is_not_null()).select(
        pl.col("id").alias("room_id"),
        pl.col("coords_lat").alias("room_lat"),
        pl.col("coords_lon").alias("room_lon"),
    )
    poi_needs_coords = df.filter((pl.col("type") == "poi") & pl.col("coords_lat").is_null()).select(
        "id",
        pl.col("parents").list.last().alias("direct_parent"),
    )
    if poi_needs_coords.height > 0:
        poi_room_match = poi_needs_coords.join(
            room_coords, left_on="direct_parent", right_on="room_id", how="inner"
        ).select("id", "room_lat", "room_lon")
        if poi_room_match.height > 0:
            df = df.join(poi_room_match, on="id", how="left")
            df = df.with_columns(
                pl.coalesce(pl.col("coords_lat"), pl.col("room_lat")).alias("coords_lat"),
                pl.coalesce(pl.col("coords_lon"), pl.col("room_lon")).alias("coords_lon"),
                pl.when(pl.col("coords_lat").is_null() & pl.col("room_lat").is_not_null())
                .then(pl.lit("inferred"))
                .otherwise(pl.col("coords_source"))
                .alias("coords_source"),
                pl.when(pl.col("coords_lat").is_null() & pl.col("room_lat").is_not_null())
                .then(pl.lit("building"))
                .otherwise(pl.col("coords_accuracy"))
                .alias("coords_accuracy"),
            ).drop("room_lat", "room_lon")

    # 3b. For rooms/virtual_rooms/poi still without coords: copy from parent building
    building_coords = df.filter(pl.col("type") == "building").select(
        pl.col("id").alias("bldg_id"),
        pl.col("coords_lat").alias("bldg_lat"),
        pl.col("coords_lon").alias("bldg_lon"),
    )

    needs_coords = df.filter(
        pl.col("type").is_in(["room", "virtual_room", "poi"]) & pl.col("coords_lat").is_null()
    ).select("id", "parents")

    if needs_coords.height > 0:
        parent_buildings = needs_coords.explode("parents").rename({"parents": "parent_id"})
        parent_buildings = parent_buildings.join(building_coords, left_on="parent_id", right_on="bldg_id", how="inner")

        # Check for entries without exactly one building parent
        parent_counts = parent_buildings.group_by("id").len()
        bad_counts = parent_counts.filter(pl.col("len") != 1)
        if bad_counts.height > 0:
            for row in bad_counts.iter_rows(named=True):
                _logger.error(f"Could not find distinct parent building for {row['id']}")
            error = True

        no_parent = needs_coords.join(parent_buildings.select("id").unique(), on="id", how="anti")
        if no_parent.height > 0:
            for rid in no_parent["id"].to_list():
                _logger.error(f"Could not find distinct parent building for {rid}")
            error = True

        parent_buildings = parent_buildings.group_by("id").agg(
            pl.col("bldg_lat").first(),
            pl.col("bldg_lon").first(),
        )

        df = df.join(parent_buildings, on="id", how="left")
        df = df.with_columns(
            pl.coalesce(pl.col("coords_lat"), pl.col("bldg_lat")).alias("coords_lat"),
            pl.coalesce(pl.col("coords_lon"), pl.col("bldg_lon")).alias("coords_lon"),
            pl.when(pl.col("coords_lat").is_null() & pl.col("bldg_lat").is_not_null())
            .then(pl.lit("inferred"))
            .otherwise(pl.col("coords_source"))
            .alias("coords_source"),
            pl.when(pl.col("coords_lat").is_null() & pl.col("bldg_lat").is_not_null())
            .then(pl.lit("building"))
            .otherwise(pl.col("coords_accuracy"))
            .alias("coords_accuracy"),
        ).drop("bldg_lat", "bldg_lon")

    # 4. For areas/campuses/sites/joined_buildings without coords: average of children buildings
    needs_avg = df.filter(
        pl.col("type").is_in(["site", "area", "campus", "joined_building"]) & pl.col("coords_lat").is_null()
    ).select("id", "children_flat")

    if needs_avg.height > 0:
        no_children = needs_avg.filter(pl.col("children_flat").is_null() | (pl.col("children_flat").list.len() == 0))
        if no_children.height > 0:
            for rid in no_children["id"].to_list():
                _logger.error(f"Cannot infer coordinate of '{rid}' because it has no children")
            error = True

        exploded = (
            needs_avg.filter(pl.col("children_flat").is_not_null() & (pl.col("children_flat").list.len() > 0))
            .explode("children_flat")
            .rename({"children_flat": "child_id"})
        )

        if exploded.height > 0:
            exploded = exploded.join(building_coords, left_on="child_id", right_on="bldg_id", how="inner")
            avg_coords = exploded.group_by("id").agg(
                pl.col("bldg_lat").mean().alias("avg_lat"),
                pl.col("bldg_lon").mean().alias("avg_lon"),
            )

            df = df.join(avg_coords, on="id", how="left")
            df = df.with_columns(
                pl.coalesce(pl.col("coords_lat"), pl.col("avg_lat")).alias("coords_lat"),
                pl.coalesce(pl.col("coords_lon"), pl.col("avg_lon")).alias("coords_lon"),
                pl.when(pl.col("coords_source").is_null() & pl.col("avg_lat").is_not_null())
                .then(pl.lit("inferred"))
                .otherwise(pl.col("coords_source"))
                .alias("coords_source"),
            ).drop("avg_lat", "avg_lon")

    # 5. Check for entries of unknown type that have no coords
    unknown = df.filter(
        (pl.col("type") != "root")
        & pl.col("coords_lat").is_null()
        & ~pl.col("type").is_in(
            ["room", "virtual_room", "poi", "site", "area", "campus", "joined_building", "building"]
        )
    )
    if unknown.height > 0:
        for row in unknown.iter_rows(named=True):
            _logger.error(f"Don't know how to infer coordinate for entry type '{row['type']}'")
        error = True

    if error:
        raise RuntimeError("Aborting due to errors")

    return df


def check_coords(df: pl.DataFrame) -> None:
    """Check for issues with coordinates."""
    non_root = df.filter(pl.col("type") != "root")

    # Check missing lat/lon
    bad = non_root.filter(pl.col("coords_lat").is_null() | pl.col("coords_lon").is_null())
    if bad.height > 0:
        ids = bad["id"].to_list()
        raise RuntimeError(
            f"The following entries do not have proper coordinates assigned: {ids}. "
            "Please provide accurate coordinates!"
        )

    bad_zero = non_root.filter((pl.col("coords_lat") == 0.0) | (pl.col("coords_lon") == 0.0))
    if bad_zero.height > 0:
        row = bad_zero.row(0, named=True)
        raise RuntimeError(f"{row['id']}: lat and/or lon coordinate is zero. Please provide an accurate coordinate!")

    # Check UTM zero values
    bad_utm = non_root.filter(
        pl.col("coords_utm_easting").is_not_null()
        & ((pl.col("coords_utm_easting") == 0.0) | (pl.col("coords_utm_northing") == 0.0))
    )
    if bad_utm.height > 0:
        row = bad_utm.row(0, named=True)
        raise RuntimeError(
            f"{row['id']}: utm coordinate is zero. There is very likely an error in the source data "
            f"(UTM coordinates are either from the Roomfinder or automatically calculated).",
        )


def validate_coords(df: pl.DataFrame) -> None:
    """Check that room coordinates are not too far away from their parent."""
    rooms = df.filter(pl.col("type") == "room").select(
        "id",
        "coords_lat",
        "coords_lon",
        pl.col("parents").list.last().alias("parent_id"),
    )
    parent_coords = df.select(
        pl.col("id").alias("p_id"),
        pl.col("coords_lat").alias("p_lat"),
        pl.col("coords_lon").alias("p_lon"),
    )
    rooms = rooms.join(parent_coords, left_on="parent_id", right_on="p_id", how="left")

    rooms = rooms.with_columns(
        pl.struct("coords_lat", "coords_lon", "p_lat", "p_lon")
        .map_elements(
            lambda s: distance_via_great_circle(s["coords_lat"], s["coords_lon"], s["p_lat"], s["p_lon"]),
            return_dtype=pl.Float64,
        )
        .alias("distance")
    )

    too_far = rooms.filter(pl.col("distance") > MAX_DISTANCE_METERS_FROM_PARENT)
    if too_far.height > 0:
        details = [
            f"{row['id']} is {row['distance']:.0f}m away from its parent {row['parent_id']}"
            for row in too_far.iter_rows(named=True)
        ]
        raise RuntimeError(
            "The following rooms are too far away from their parent building. "
            "Please recheck if the coordinates make sense:\n" + "\n".join(details)
        )


def add_and_check_coords(df: pl.DataFrame) -> pl.DataFrame:
    """Add coordinates to all entries and check for issues."""
    assert_buildings_have_coords(df)
    df = assign_coordinates(df)
    check_coords(df)
    validate_coords(df)
    return df
