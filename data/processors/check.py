

def check_coords(input_data):
    """ Check for issues with coordinates """
    error = False

    for iid, data in input_data.items():
        if obj["type"] in {"site", "campus", "area", "joined_building", "building"}:
            if "coords" not in data or \
               "lat" not in data["coords"] or "lon" not in data["coords"]:
                print(f"{iid}: No lat/lon coordinate found for this area / building")
                error = True
                continue

        if data["coords"]["lat"] == 0. or data["coords"]["lon"] == 0.:
            print(f"{iid}: lat and/or lon coordinate is zero. "
                   "If coordinate is unknown leave this field out.")
            error = True
            continue

        if "utm" in data["coords"] and \
           (data["coords"]["utm"]["easting"] == 0. or data["coords"]["utm"]["northing"] == 0.):
            print(f"{iid}: utm coordinat is zero. "
                   "If coordinate is unknown leave this field out.")
            error = True
            continue
