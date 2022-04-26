import json
import math
import copy
import os.path
from typing import Any

import utm
import yaml
from PIL import Image
from pathlib import Path

EXTERNAL_PATH = Path(__file__).parent.parent / "external"
RF_MAPS_PATH = EXTERNAL_PATH / "maps" / "roomfinder"


def assign_coordinates(data):
    """
    Assign coordinates to all entries (except root) and make sure they match the data format.
    """
    # TODO: In the future we might calculate the coordinates from OSM data
    
    # The inference of coordinates in this function for all entries is based on the
    # coordinates of buildings, so it is necessary, that at least all buildings have
    # a coordinate.
    buildings_without_coord = set()
    for _id, entry in data.items():
        if entry["type"] == "building":
            if "coords" not in entry:
                buildings_without_coord.add(entry["id"])
    if len(buildings_without_coord) > 0:
        raise RuntimeError(f"Error: No coordinates known for the following buildings: "
              f"{buildings_without_coord}")
    
    # All errors are collected first before quitting in the end if any
    # error occured.
    error = False
    
    for _id, entry in data.items():
        if entry["type"] == "root":
            continue
        
        if "coords" in entry:
            # While not observed so far, coordinate values of zero are typical for missing
            # data so we check this here.
            if entry["coords"].get("lat", None) == 0. or entry["coords"].get("lon", None) == 0.:
                print(f"Error: Lat and/or lon coordinate is zero for '{_id}': "
                      f"{entry['coords']}")
                error = True
                continue
            if "utm" in entry["coords"] \
                    and (entry["coords"]["utm"]["easting"] == 0.
                         or entry["coords"]["utm"]["northing"] == 0.):
                print(f"Error: UTM coordinate is zero for '{_id}': "
                      f"{entry['coords']}")
                error = True
                continue
            
            # Convert between utm and lat/lon if necessary
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
                latlon_coord = utm.to_latlon(utm_coord["easting"], utm_coord["northing"],
                                             utm_coord["zone_number"], utm_coord["zone_letter"])
                entry["coords"]["lat"] = latlon_coord[0]
                entry["coords"]["lat"] = latlon_coord[1]
            
            # If no source is provided, "navigatum" is assumed because Roomfinder
            # provided coordinates will have "roomfinder" set.
            if "source" not in entry["coords"]:
                entry["coords"]["source"] = "navigatum"
        else:
            # For rooms we check whether its parent has a coordinate
            if entry["type"] in {"room", "virtual_room"}:
                building_parent = list(filter(lambda e: data[e]["type"] == "building",
                                              entry["parents"]))
                if len(building_parent) != 1:
                    print(f"Error: Could not find distinct parent building for {_id}")
                    error = True
                    continue
                building_parent = data[building_parent[0]]
                
                # Copy probably not required, but this could avoid unwanted side effects
                entry["coords"] = copy.deepcopy(building_parent["coords"])
                entry["coords"]["accuracy"] = "building"
                entry["coords"]["source"] = "inferred"
            elif entry["type"] in {"site", "area", "campus", "joined_building"}:
                # Calculate the average coordinate of all children buildings
                # TODO: garching-interims
                if "children_flat" not in entry:
                    print(f"Error: Cannot infer coordinate of '{_id}' because it has no children")
                    error = True
                    continue
                
                lats, lons = ([], [])
                for c in entry["children_flat"]:
                    if data[c]["type"] == "building":
                        lats.append(data[c]["coords"]["lat"])
                        lons.append(data[c]["coords"]["lon"])
                lat_coord = sum(lats) / len(lats)
                lon_coord = sum(lons) / len(lons)
                utm_coord = utm.from_latlon(lat_coord, lon_coord)
                entry["coords"] = {
                    "lat": lat_coord,
                    "lon": lon_coord,
                    "utm": {
                        "zone_number": utm_coord[2],
                        "zone_letter": utm_coord[3],
                        "easting": utm_coord[0],
                        "northing": utm_coord[1],
                    },
                    "source": "inferred"
                }
            else:
                print(f"Error: Don't know how to infer coordinate for entry type '{entry['type']}'")
                error = True
                continue
    
    if error:
        raise RuntimeError("Aborting due to errors")


def assign_roomfinder_maps(data):
    """
    Assign roomfinder maps to all entries if there are none yet specified.
    """
    maps_list = _load_maps_list()

    # There are also Roomfinder-like custom maps, that we assign here
    custom_maps = _load_custom_maps()

    for _id, entry in data.items():
        if entry["type"] == "root":
            continue

        if entry.get("maps", {}).get("roomfinder", {}).get("available", []):
            continue

        # Use maps from parent building, if there is no precise coordinate known
        entry_accuracy_building = entry["coords"].get("accuracy", None) == "building"
        if entry_accuracy_building and entry["type"] in {"room", "virtual_room"}:
            _set_maps_from_parent(data, entry)
            continue

        available_maps = _extract_available_maps(entry, custom_maps, maps_list)

        # For entries of these types only show maps that contain all (direct) children.
        # This is to make sure that only (high scale) maps are included here that make sense.
        # TODO: zentralgelaende
        if entry["type"] in {"site", "campus", "area", "joined_building", "building"} \
                and "children" in entry:
            for m in available_maps:
                for c in entry["children"]:
                    lat_coord, lon_coord = data[c]["coords"]["lat"], data[c]["coords"]["lon"]
                    if not (m["latlonbox"]["south"] < lat_coord < m["latlonbox"]["north"] and
                            m["latlonbox"]["west"] < lon_coord < m["latlonbox"]["east"]):
                        available_maps.remove(m)
                        break

        if not available_maps:
            print(f"Warning: No Roomfinder maps available for '{_id}'")
            continue
        _save_map_data(available_maps, entry)


def _save_map_data(available_maps, entry):
    # Select map with lowest scale as default
    default_map = available_maps[0]
    for m in available_maps:
        if int(m["scale"]) < int(default_map["scale"]):
            default_map = m

    roomfinder_map_data = {
        "default": default_map["id"],
        "available": [
            {
                "id": m["id"],
                "scale": m["scale"],
                "name": m["desc"],
                "width": m["width"],
                "height": m["height"],
                "source": m.get("source", "Roomfinder"),
                "path": m.get("path", f"webp/{m['id']}.webp")
            }
            for m in available_maps
        ],
    }
    entry.setdefault("maps", {})["roomfinder"] = roomfinder_map_data


def _set_maps_from_parent(data, entry):
    building_parent = [e for e in entry["parents"] if data[e]["type"] == "building"]
    # Verification of this already done for coords, see above
    building_parent = data[building_parent[0]]
    if not building_parent.get("maps", {}) \
            .get("roomfinder", {}) \
            .get("available", []):
        roomfinder_map_data = {"is_only_building": True}
        # Both share the reference now, assuming that the parening_parent
        # will be processed some time later in this loop.
        building_parent.setdefault("maps", {})["roomfinder"] = roomfinder_map_data
        entry.setdefault("maps", {})["roomfinder"] = roomfinder_map_data
    else:
        # TODO: 5510.02.001
        roomfinder_map_data = entry.setdefault("maps", {}).get("roomfinder", {})
        roomfinder_map_data.update(building_parent["maps"]["roomfinder"])
        roomfinder_map_data["is_only_building"] = True


def _extract_available_maps(entry, custom_maps, maps_list):
    available_maps = []
    for (b_id, floor), m in custom_maps.items():
        if entry["type"] == "room" and b_id in entry["parents"] and \
                "tumonline_data" in entry and f".{floor}." in entry["tumonline_data"]["roomcode"]:
            available_maps.append(m)
    lat_coord, lon_coord = (entry["coords"]["lat"], entry["coords"]["lon"])
    for m in maps_list:
        # Assuming coordinates in Central Europe
        if m["latlonbox"]["south"] < lat_coord < m["latlonbox"]["north"] and \
                m["latlonbox"]["west"] < lon_coord < m["latlonbox"]["east"]:
            available_maps.append(m)
    # TODO: Sort
    return available_maps


def _merge_str(s1: str, s2: str):
    """Merges two strings. The Result is of the format common_prefix s1/s2 common_suffix"""
    if s1 == s2:
        return s1
    prefix = os.path.commonprefix((s1, s2))
    suffix = os.path.commonprefix((s1[::-1], s2[::-1]))[::-1]
    s1 = s1.removeprefix(prefix).removesuffix(suffix)
    s2 = s2.removeprefix(prefix).removesuffix(suffix)
    return prefix + s1 + "/" + s2 + suffix


def _merge_maps(map1, map2):
    """Merges two Maps into one merged map"""
    result_map = {}
    for key in map1.keys():
        if key == "id":
            result_map["id"] = map1["id"]
        elif isinstance(map1[key], dict):
            result_map[key] = _merge_maps(map1[key], map2[key])
        elif isinstance(map1[key], str):
            result_map[key] = _merge_str(map1[key], map2[key])
        elif isinstance(map1[key], int) or isinstance(map1[key], float):
            result_map[key] = sum((map1[key], map2[key])) / 2
        else:
            values = map1[key]
            raise NotImplementedError(f"the {key=} of with {type(values)=} does not have a merging-operation defined")
    return result_map


def _deduplicate_maps(maps_list):
    """Remove content 1:1 duplicates from the maps_list"""
    content_to_filename_dict = {}
    file_renaming_table: dict[str, str] = dict()
    for filename in RF_MAPS_PATH.iterdir():
        with open(filename, "rb") as file:
            content = file.read()
            _id = filename.with_suffix("").name
            if content in content_to_filename_dict:
                file_renaming_table[_id] = content_to_filename_dict[content]
            else:
                content_to_filename_dict[content] = _id
    # we merge the maps into the first occurrence of said map.
    filtered_map = {m["id"]: m for m in maps_list}
    for m in maps_list:
        _id = m["id"]
        if _id in file_renaming_table:
            map1 = filtered_map.pop(_id)
            map2 = filtered_map[file_renaming_table[_id]]
            if filtered_map[file_renaming_table[_id]] != map1:
                filtered_map[file_renaming_table[_id]] = _merge_maps(map1, map2)
    maps_list = list(filtered_map.values())
    return maps_list


def _load_maps_list():
    """Read the Roomfinder maps. The world-map is not used"""
    with open("external/maps_roomfinder.json") as f:
        maps_list: list[dict[str, Any]] = json.load(f)
    world_map = None
    for m in maps_list:
        if m["id"] == 9:  # World map is not used
            world_map = m
        else:
            m["latlonbox"]["north"] = float(m["latlonbox"]["north"])
            m["latlonbox"]["south"] = float(m["latlonbox"]["south"])
            m["latlonbox"]["east"] = float(m["latlonbox"]["east"])
            m["latlonbox"]["west"] = float(m["latlonbox"]["west"])
            m["id"] = f"rf{m['id']}"
    maps_list.remove(world_map)

    # remove 1:1 content duplicates
    maps_list = _deduplicate_maps(maps_list)

    return maps_list


def build_roomfinder_maps(data):
    """ Generate the map information for the Roomfinder maps. """
    
    # Read the Roomfinder and custom maps
    with open("external/maps_roomfinder.json") as f:
        maps_list = json.load(f)
    custom_maps = _load_custom_maps()
    
    # For each map, we calculate the boundaries in UTM beforehand
    maps = {}
    for m in maps_list + list(custom_maps.values()):
        if "latlonbox" in m:
            latlonbox = m["latlonbox"]
            
            latlonbox["north_west"] = (float(latlonbox["north"]), float(latlonbox["west"]))
            latlonbox["south_east"] = (float(latlonbox["south"]), float(latlonbox["east"]))

            # Roomfinder data is with ints as id, but we use a string based format
            if isinstance(m["id"], int):
                m["id"] = f"rf{m['id']}"
            
            maps[m["id"]] = m

    for _id, entry in data.items():
        if len(entry.get("maps", {}).get("roomfinder", {}).get("available", [])) > 0:
            world_map = None
            for entry_map in entry["maps"]["roomfinder"]["available"]:
                # The world map (id 9) is currently excluded, because it would need a different
                # projection treatment.
                if entry_map["id"] == "rf9":
                    world_map = entry_map
                    continue
                
                m = maps[entry_map["id"]]
                box = m["latlonbox"]
                
                # For the map regions used we can assume that the lat/lon graticule is
                # rectangular within that map. It is however not square (roughly 2:3 aspect),
                # so for simplicity we first map it into the cartesian pixel coordinate
                # system of the image and then apply the rotation.
                # Note: x corrsp. to longitude, y corresp. to latitude
                ex, ey = (entry["coords"]["lon"],
                          entry["coords"]["lat"])
                
                box_delta_x = abs(box["north_west"][1] - box["south_east"][1])
                box_delta_y = abs(box["north_west"][0] - box["south_east"][0])
                
                rel_x, rel_y = (abs(box["north_west"][1] - ex) / box_delta_x,
                                abs(box["north_west"][0] - ey) / box_delta_y)
                
                ix0, iy0 = (rel_x * entry_map["width"],
                            rel_y * entry_map["height"])
                
                cx, cy = (entry_map["width"] / 2,
                          entry_map["height"] / 2)
                
                angle = math.radians(float(box["rotation"]))
                ix, iy = (cx + (ix0 - cx) * math.cos(angle) - (iy0 - cy) * math.sin(angle),
                          cy + (ix0 - cx) * math.sin(angle) + (iy0 - cy) * math.cos(angle))

                # The result is the position of the pixel in this image corresponding
                # to the coordinate.
                entry_map["x"] = round(ix)
                entry_map["y"] = round(iy)

                # Finally, set source and filepath so that they are available for all maps
                entry_map.setdefault("source", "Roomfinder")
                entry_map.setdefault("path", f"webp/{entry_map['id']}.webp")

            if world_map is not None:
                entry["maps"]["roomfinder"]["available"].remove(world_map)



def _load_custom_maps():
    """ Load the custom maps like Roomfinder maps """
    with open("sources/45_custom-maps.yaml") as f:
        custom_maps = yaml.safe_load(f.read())
        
    # Convert into the format used by maps_roomfinder.json:
    maps_out = {}
    for map_group in custom_maps:
        base_data = {
            "source": map_group["props"].get("source", "NavigaTUM-Contributors"),
            # For some reason, these are given as str
            "scale": str(map_group["props"]["scale"]),
            "latlonbox": {
                "north":    map_group["props"]["north"],
                "east":     map_group["props"]["east"],
                "west":     map_group["props"]["west"],
                "south":    map_group["props"]["south"],
                "rotation": map_group["props"]["rotation"],
            }
        }
        for sub_map in map_group["maps"]:
            img = Image.open("sources/img/maps/roomfinder/webp/" + sub_map["file"])
            maps_out[(sub_map["b_id"], sub_map["floor"])] = {
                "desc": sub_map["desc"],
                "id": ".".join(sub_map["file"].split(".")[:-1]),
                "path": "webp/" + sub_map["file"],
                "width": img.width,
                "height": img.height,
                **base_data
            }

    return maps_out
