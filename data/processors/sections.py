
def compute_props(data):
    """
    Create the "computed" value in "props".
    """
    for _id, entry in data.items():
        if "props" in entry:
            props = entry["props"]
            computed = []
            
            if "ids" in props:
                if "b_id" in props["ids"]:
                    computed.append({"Gebäudekennung": props["ids"]["b_id"]})
                if "roomcode" in props["ids"]:
                    computed.append({"Raumkennung": props["ids"]["roomcode"]})
                if "arch_name" in props["ids"]:
                    computed.append({"Architekten-Name": props["ids"]["arch_name"].split("@")[0]})
            if "b_prefix" in entry and entry["b_prefix"] != _id:
                b_prefix = [entry["b_prefix"]] if type(entry["b_prefix"]) is str else entry["b_prefix"]
                computed.append({"Gebäudekennungen": ", ". join([
                    p.ljust(4, "x")
                    for p in b_prefix
                ])})
            if "address" in props:
                computed.append({"Adresse": "{}, {}".format(
                    props["address"]["street"],
                    props["address"]["plz_place"]
                )})
            if "stats" in props:
                if "n_buildings" in props["stats"]:
                    computed.append({"Anzahl Gebäude": str(props["stats"]["n_buildings"])})
                if "n_rooms" in props["stats"]:
                    if props["stats"]["n_rooms"] == props["stats"]["n_rooms_reg"]:
                        computed.append({"Anzahl Räume": str(props["stats"]["n_rooms"])})
                    else:
                        computed.append({"Anzahl Räume": "{} ({} ohne Flure etc.)".format(
                            props["stats"]["n_rooms"],
                            props["stats"]["n_rooms_reg"]
                        )})
                if "n_seats" in props["stats"]:
                    computed.append({"Sitzplätze": str(props["stats"]["n_seats"])})
            if "generic" in props:
                for e in props["generic"]:
                    if type(e[1]) is dict:
                        computed.append({"name": e[0], **e[1]})
                    else:
                        computed.append({"name": e[0], "text": e[1]})
            
            # Reformat if required (just to have less verbosity in the code above)
            for i, c in enumerate(computed):
                if "name" not in c:
                    computed[i] = {"name": list(c.keys())[0], "text": list(c.values())[0]}
            
            entry["props"]["computed"] = computed
            

def generate_buildings_overview(data):
    """
    Generate the "buildings_overview" section
    """
    for _id, entry in data.items():
        if entry["type"] not in {"area", "site", "campus"} or \
           "children_flat" not in entry:
            continue
        
        if "buildings_overview" in entry.get("generators", {}):
            options = entry["generators"]["buildings_overview"]
        else:
            options = {"n_visible": 6, "list_start": []}
        
        # Collect buildings to display for this entry.
        buildings = []
        for child_id in entry["children"]:
            child = data[child_id]
            if child["type"] in {"area", "site", "campus", "building", "joined_building"}:
                buildings.append(child)
        # for child_id in entry["children_flat"]:
        #    child = data[child_id]
        #    if child["type"] == "joined_building" or \
        #       (child["type"] == "building"
        #        and data[child["parents"][-1]]["type"] != "joined_building"):
        #        buildings.append(child)
        # Entries are sorted alphabetically in second order to be predictable
        buildings = list(sorted(
            buildings,
            key=lambda e: str(len(e.get("children_flat", []))).zfill(5) + e["name"],
            reverse=True)
        )
        
        # The "list_start" can overwrite how the list of buildings starts,
        # and optionally also add other entries. All other entries are appended
        # after them.
        merged_ids = options["list_start"] + \
                     [b["id"] for b in buildings if b["id"] not in options["list_start"]]
        
        b_overview = entry.setdefault("sections", {}).setdefault("buildings_overview", {})
        b_overview["n_visible"] = options["n_visible"]
        b_overview["entries"] = []
        for child_id in merged_ids:
            try:
                child = data[child_id]
            except KeyError:
                raise RuntimeError(f"Error: Unknown id '{child_id}' found when generating buildings_overview for '{_id}'")
            
            if child["type"] in {"building", "joined_building"}:
                n_rooms = child["props"]["stats"].get("n_rooms", 0)
                if n_rooms == 0:
                    subtext = "Keine Räume bekannt"
                else:
                    subtext = "{} Räume".format(n_rooms)
            elif child["type"] == "area":
                subtext = "{} Gebäude, {} Räume".format(
                    child["props"]["stats"].get("n_buildings", 0),
                    child["props"]["stats"].get("n_rooms", 0),
                )
            elif child["type"] == "site":
                subtext = "{} Gebäude, {} Räume (Außenstelle)".format(
                    child["props"]["stats"].get("n_buildings", 0),
                    child["props"]["stats"].get("n_rooms", 0),
                )
            else:
                raise RuntimeError(f"Error: Cannot generate buildings_overview subtext "
                                   f"for type '{child['type']}', for: '{_id}', child id: '{child_id}'")

            b_overview["entries"].append({
                "id": child_id,
                "name": child["short_name"] if "short_name" in child else child["name"],
                "subtext": subtext,
                "thumb": child["img"][0]["name"] if child.get("img", []) else None,
            })
        
        
def generate_rooms_overview(data):
    """
    Generate the "rooms_overview" section
    """
    for _id, entry in data.items():
        # if entry["type"] not in {"building", "joined_building", "virtual_room"} or \
        if entry["type"] not in {"area", "site", "campus", "building", "joined_building", "virtual_room"} or \
           "children_flat" not in entry:
            continue

        rooms = {}
        for child_id in entry["children_flat"]:
            child = data[child_id]
            if child["type"] == "room":
                usage = child["usage"] if "usage" in child else {"name": "unbekannt"}
                rooms.setdefault(usage["name"], []).append({
                    "id": child_id,
                    "name": child["name"],
                })

        r_overview = entry.setdefault("sections", {}).setdefault("rooms_overview", {})
        r_overview["usages"] = [
            {
                "name": u[0],
                "count": len(u[1]),
                "children": list(sorted(u[1], key=lambda r: r["name"])),
            }
            for u in sorted(rooms.items(), key=lambda e: e[0])
        ]
