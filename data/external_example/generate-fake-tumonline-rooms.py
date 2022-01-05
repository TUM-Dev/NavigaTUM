# This code generates fake TUMOnline room data, so that the compilation
# works even if only Roomfinder data is available.
# This is only intended for testing purposes, any might break as data changes.

import json


with open("rooms_roomfinder.json") as f:
    rooms = json.load(f)


# Add data that is expected by 02_rooms-extended.yaml
rooms.extend([
    {
        "b_id": "0509",
        "r_id": "980@0509",
        "r_level": "0",
        "r_alias": "",
    },
    {
        "b_id": "5101",
        "r_id": "257@5101",
        "r_level": "0",
        "r_alias": "",
    },
    {
        "b_id": "5606",
        "r_id": "036@5606",
        "r_level": "0",
        "r_alias": "",
    },
])

tumonline_data = []
for room in rooms:
    # Skip non-room entries in the Roomfinder
    if room["b_id"] == "0000":
        continue
    
    # The 'SAP ID' is generated so that it matches the common
    # room ID format. It might be different to the real ID,
    # which is not stored in the Roomfinder.
    # Matching Roomfinder and TUMOnline data is based on the
    # archname, so this is not a problem for testing.
    sap_id = "{}.{}.{}".format(room["b_id"],
                               room["r_level"].zfill(2) if room["r_level"] != "0" else "EG",
                               room["r_id"].split("@")[0].replace(".", "-"))
    
    # Known patches (see 16_roomfinder-merge-patches.yaml)
    if room["r_id"] == "-1519@0505":
        sap_id = "0505.U1.519"
    if room["r_id"] == "-1530@0505":
        sap_id = "0505.U1.530"
    
    tumonline_data.append({
        "roomcode": sap_id,
        "room_link": "wbRaum.editRaum?pRaumNr=42",
        "b_filter_id": 0,
        "b_area_id": 0,
        "arch_name": room["r_id"],
        "alt_name": room["r_alias"],
        "address": "Arcisstraße.   21, EG",
        "address_link": "ris.einzelraum?raumkey=42",
        "plz_place": "80333 München",
        "operator": "[ TUZVZA4 ]",
        "op_link": "webnav.navigate_to?corg=42",
        "calendar": None,
        "usage": 0,
    })


with open("rooms_tumonline.json", "w") as f:
    json.dump(tumonline_data, f)
