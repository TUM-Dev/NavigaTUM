import copy
import logging
from typing import Any

import utm
from utils import distance_via_great_circle

MAX_DISTANCE_METERS_FROM_PARENT = 250


def assert_buildings_have_coords(data: dict[str, dict[str, Any]]) -> None:
    """
    Assert that all buildings have coordinates

    The inference of coordinates in further functions for all entries is based on the
    coordinates of buildings, so it is necessary, that at least all buildings have
    a coordinate.
    """
    buildings = [(_id, entry) for _id, entry in data.items() if entry["type"] == "building"]
    if buildings_without_coord := {_id for _id, entry in buildings if "coords" not in entry}:
        raise RuntimeError(
            "No coordinates known for the following buildings:\n"
            + "\n".join([f"{_id}: {data[_id]['name']}" for _id in sorted(buildings_without_coord)]),
        )


def assign_coordinates(data: dict[str, dict[str, Any]]) -> None:
    """Assign coordinates to all entries (except root) and make sure they match the data format."""
    # TODO: In the future we might calculate the coordinates from OSM data

    error = False

    for _id, entry in data.items():
        if entry["type"] == "root":
            continue

        if "coords" in entry:
            _convert_coordinate_formats(entry)

            # If no source is provided, "navigatum" is assumed because Roomfinder
            # provided coordinates will have "roomfinder" set.
            if "source" not in entry["coords"]:
                entry["coords"]["source"] = "navigatum"
        else:
            # For rooms & POIs, we check whether its parent has a coordinate
            if entry["type"] in {"room", "virtual_room", "poi"}:
                building_parent = [data[e] for e in entry["parents"] if data[e]["type"] == "building"]
                if len(building_parent) != 1:
                    logging.error(f"Could not find distinct parent building for {_id}")
                    error = True
                    continue
                entry["coords"] = _get_coordinte_from_parent(building_parent[0])
            elif entry["type"] in {"site", "area", "campus", "joined_building"}:
                # Calculate the average coordinate of all children buildings
                # TODO: garching-interims
                if "children_flat" not in entry:
                    logging.error(f"Cannot infer coordinate of '{_id}' because it has no children")
                    error = True
                    continue

                entry["coords"] = _calc_coordinte_from_children(data, entry)
            else:
                logging.error(f"Don't know how to infer coordinate for entry type '{entry['type']}'")
                error = True
                continue

    if error:
        raise RuntimeError("Aborting due to errors")


def _get_coordinte_from_parent(building_parent):
    """Copy probably not required, but this could avoid unwanted side effects"""
    coords = copy.deepcopy(building_parent["coords"])
    coords["accuracy"] = "building"
    coords["source"] = "inferred"
    return coords


def _convert_coordinate_formats(entry):
    """Convert between utm and lat/lon if necessary"""
    if "lat" not in entry["coords"]:
        utm_coord = entry["coords"]["utm"]
        latlon_coord = utm.to_latlon(
            utm_coord["easting"],
            utm_coord["northing"],
            utm_coord["zone_number"],
            utm_coord["zone_letter"],
        )
        entry["coords"]["lat"] = latlon_coord[0]
        entry["coords"]["lat"] = latlon_coord[1]


def _calc_coordinte_from_children(data, entry):
    """Calculate the average coordinate of all children"""
    lats, lons = ([], [])
    for child in entry["children_flat"]:
        if data[child]["type"] == "building":
            lats.append(data[child]["coords"]["lat"])
            lons.append(data[child]["coords"]["lon"])
    lat_coord = sum(lats) / len(lats)
    lon_coord = sum(lons) / len(lons)
    return {
        "lat": lat_coord,
        "lon": lon_coord,
        "source": "inferred",
    }


def check_coords(input_data):
    """Check for issues with coordinates"""
    for iid, data in input_data.items():
        if data["type"] == "root":
            continue
        if "coords" not in data or "lat" not in data["coords"] or "lon" not in data["coords"]:
            raise RuntimeError(
                f"{iid}: Does not have proper coordinates assinged. Please provide an accurate coordinate!"
            )

        if data["coords"]["lat"] == 0.0 or data["coords"]["lon"] == 0.0:
            raise RuntimeError(f"{iid}: lat and/or lon coordinate is zero. Please provide an accurate coordinate!")

        if (utm_coord := data["coords"].get("utm")) and (utm_coord["easting"] == 0.0 or utm_coord["northing"] == 0.0):
            raise RuntimeError(
                f"{iid}: utm coordinate is zero. There is very likely an error in the source data "
                f"(UTM coordinates are either from the Roomfinder or automatically calculated).",
            )


def validate_coords(input_data):
    """Check that coordinates are not too far away from their parent"""
    for iid, data in input_data.items():
        if data["type"] != "room":
            continue
        coords = data["coords"]
        parent_id = data["parents"][-1]
        parent_coords = input_data[parent_id]["coords"]

        distance_to_parent = distance_via_great_circle(
            coords["lat"],
            coords["lon"],
            parent_coords["lat"],
            parent_coords["lon"],
        )

        if distance_to_parent > MAX_DISTANCE_METERS_FROM_PARENT:
            raise RuntimeError(
                f"{iid} {coords} is {distance_to_parent}m away from its parent {parent_id} {parent_coords}. "
                "Please recheck if the coordinate makes sense",
            )


def add_and_check_coords(data: dict[str, dict[str, Any]]) -> None:
    """Add coordinates to all entries and check for issues"""
    assert_buildings_have_coords(data)
    assign_coordinates(data)
    check_coords(data)
    validate_coords(data)
