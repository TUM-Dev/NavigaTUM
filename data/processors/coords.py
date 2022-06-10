import copy
import logging

import utm  # type: ignore


def assert_buildings_have_coords(data):
    """
    The inference of coordinates in further functions for all entries is based on the
    coordinates of buildings, so it is necessary, that at least all buildings have
    a coordinate.
    """
    buildings = [(_id, entry) for _id, entry in data.items() if entry["type"] == "building"]
    buildings_without_coord = {_id for _id, entry in buildings if "coords" not in entry}
    if buildings_without_coord:
        raise RuntimeError(f"No coordinates known for the following buildings: {buildings_without_coord}")


def assign_coordinates(data):
    """
    Assign coordinates to all entries (except root) and make sure they match the data format.
    """
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
            # For rooms we check whether its parent has a coordinate
            if entry["type"] in {"room", "virtual_room"}:
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
    if "utm" not in entry["coords"]:
        utm_coord = utm.from_latlon(entry["coords"]["lat"], entry["coords"]["lon"])
        entry["coords"]["utm"] = {
            "zone_number": utm_coord[2],
            "zone_letter": utm_coord[3],
            "easting": utm_coord[0],
            "northing": utm_coord[1],
        }
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
    utm_coord = utm.from_latlon(lat_coord, lon_coord)
    return {
        "lat": lat_coord,
        "lon": lon_coord,
        "utm": {
            "zone_number": utm_coord[2],
            "zone_letter": utm_coord[3],
            "easting": utm_coord[0],
            "northing": utm_coord[1],
        },
        "source": "inferred",
    }


def check_coords(input_data):
    """Check for issues with coordinates"""

    for iid, data in input_data.items():
        if data["type"] == "root":
            continue

        if data["coords"]["lat"] == 0.0 or data["coords"]["lon"] == 0.0:
            raise RuntimeError(f"{iid}: lat and/or lon coordinate is zero. Please provide an accurate coordinate!")

        if "utm" in data["coords"] and (
            data["coords"]["utm"]["easting"] == 0.0 or data["coords"]["utm"]["northing"] == 0.0
        ):
            raise RuntimeError(
                f"{iid}: utm coordinate is zero. There is very likely an error in the source data "
                f"(UTM coordinates are either from the Roomfinder or automatically calculated).",
            )
