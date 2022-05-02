import json
import math
import copy

import utm
import yaml
from PIL import Image


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
               and (   entry["coords"]["utm"]["easting"]  == 0.
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
    # Read the Roomfinder maps
    with open("external/maps_roomfinder.json") as f:
        maps_list = json.load(f)
        
    # There are also Roomfinder-like custom maps, that we assign here
    custom_maps = _load_custom_maps()
    
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
    
    for _id, entry in data.items():
        if entry["type"] == "root":
            continue
        
        if len(entry.get("maps", {}).get("roomfinder", {}).get("available", [])) > 0:
            continue
        
        # Use maps from parent building, if there is no precise coordinate known
        if entry["type"] in {"room", "virtual_room"} and \
           entry["coords"].get("accuracy", None) == "building":
            building_parent = list(filter(lambda e: data[e]["type"] == "building",
                                            entry["parents"]))
            # Verification of this already done for coords, see above
            building_parent = data[building_parent[0]]
            
            if len(building_parent.get("maps", {})
                                  .get("roomfinder", {})
                                  .get("available", [])) == 0:
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

            continue
        
        # TODO: Sort & unique
        available_maps = []
        for (b_id, floor), m in custom_maps.items():
            if entry["type"] == "room" and b_id in entry["parents"] and \
               "tumonline_data" in entry and f".{floor}." in entry["tumonline_data"]["roomcode"]:
                available_maps.append(m)
        lat_coord, lon_coord = (entry["coords"]["lat"], entry["coords"]["lon"])
        for m in maps_list:
            # Assuming coordinates in Central Europe
            if m["latlonbox"]["south"] < lat_coord < m["latlonbox"]["north"] and \
               m["latlonbox"]["west"]  < lon_coord < m["latlonbox"]["east"]:
                available_maps.append(m)
        
        # For entries of these types only show maps that contain all (direct) children.
        # This is to make sure that only (high scale) maps are included here that make sense.
        # TODO: zentralgelaende
        if entry["type"] in {"site", "campus", "area", "joined_building", "building"} \
           and "children" in entry:
            exclude_maps = []
            for m in available_maps:
                for c in entry["children"]:
                    lat_coord, lon_coord = (data[c]["coords"]["lat"], data[c]["coords"]["lon"])
                    if not (m["latlonbox"]["south"] < lat_coord < m["latlonbox"]["north"] and
                            m["latlonbox"]["west"]  < lon_coord < m["latlonbox"]["east"]):
                        exclude_maps.append(m)
                        break
            for m in exclude_maps:
                available_maps.remove(m)
        
        if len(available_maps) == 0:
            print(f"Warning: No Roomfinder maps available for '{_id}'")
            continue
        
        # Select map with lowest scale as default
        default_map = available_maps[0]
        for m in available_maps:
            if int(m["scale"]) < int(default_map["scale"]):
                default_map = m
        
        roomfinder_map_data = {
            "default": default_map["id"],
            "available": [
                {
                    "id":     m["id"],
                    "scale":  m["scale"],
                    "name":   m["desc"],
                    "width":  m["width"],
                    "height": m["height"],
                    "source": m.get("source", "Roomfinder"),
                    "path":   m.get("path", f"webp/{m['id']}.webp")
                }
                for m in available_maps
            ],
        }
        entry.setdefault("maps", {})["roomfinder"] = roomfinder_map_data
    

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
                ix, iy = (cx + (ix0-cx)*math.cos(angle) - (iy0-cy)*math.sin(angle),
                          cy + (ix0-cx)*math.sin(angle) + (iy0-cy)*math.cos(angle))
                
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


def add_overlay_maps(data):
    """ Add the overlay maps to all entries where they apply """
    with open("sources/46_overlay-maps.yaml") as f:
        overlay_maps = yaml.safe_load(f.read())
        
    parent_lut = {m["props"]["parent"]: m for m in overlay_maps}
    parent_ids = set(parent_lut.keys())
    
    for _id, entry in data.items():
        candidates = parent_ids.intersection(entry["parents"])
        if len(candidates) > 1:
            print(f"Multiple candidates as overlay map for {_id}: {candidates}. "
                  f"Currently this is not supported! Skipping ...")
        elif bool(candidates) ^ (_id in parent_ids):  
            # either a candidate exist or _id is one of the parent ids, but not both
            overlay = parent_lut[list(candidates)[0] if len(candidates) == 1 else _id]
            overlay_data = entry.setdefault("maps", {}).setdefault("overlays", {})
            overlay_data["available"] = []
            for m in overlay["maps"]:
                overlay_data["available"].append({
                    "id": m["id"],
                    "floor": m["floor"],
                    "file": m["file"],
                    "name": m["desc"],
                    "coordinates": overlay["props"]["box"]
                })
                
                if f".{m['floor']}." in _id:
                    overlay_data["default"] = m["id"]
            
            overlay_data.setdefault("default", None)
    
        
