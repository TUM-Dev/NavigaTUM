from geopy.distance import distance

MAXDISTANCE = 1  # max distance from building

def nearby(building_coords:tuple, stations: list[dict]) -> list[dict]:
    results=[]
    for station in stations:
        distance_to_building=round(distance((station.get("lat"), station.get("lon")), (building_coords)).km,2)
        if distance_to_building <= MAXDISTANCE:
            station["lat"]=str(station.get("lat"))  #decimal to string to allow json serialization
            station["lon"]=str(station.get("lon"))
            station["distance"]=distance_to_building
            results.append(station)
    return results