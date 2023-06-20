from pathlib import Path
from external.scraping_utils import cached_json
from external.scrapers.roomfinder import scrape_maps
import csv
from decimal import Decimal
from geopy.distance import distance

# CSV indexes
STATIONID = 0
NAME = 1
ORT = 2
GLOBAL_ID = 3
WGS84X = 4  # lat
WSG84Y = 6  # lon

MAXDISTANCE = 1  # max distance from building


def avg(x1, x2):
    return (x1 + x2) / 2


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


@cached_json("public_transport.json")
def scrape_stations():
    with Path("scrapers/MVV_HSTReport2212.csv").open("r") as file:
        lines = csv.reader(file, delimiter=";")  
        next(lines) # skip first line as it contains row names
        lines = filter(lambda l: not all(not bool(i) for i in l), lines)  # filter out lines where each value is ''
        stations = list(
            map(
                lambda station: {
                    "id":station[STATIONID],
                    "name":station[NAME],
                    "ort":station[ORT],
                    "global-id":station[GLOBAL_ID],
                    "lat":Decimal(station[WGS84X].replace(",", ".")),
                    "lon":Decimal(station[WSG84Y].replace(",", ".")),
                },
                lines,
            )
        )
    buildings = scrape_maps()
    building_coords: list = []
    for building in buildings:
        # ignore entries that dont have assigned lon/lat
        if latlonbox := building.get("latlonbox"):
            # since lat and lon have rather long decimal points it is probably best to not use floats
            latn, lats = Decimal(latlonbox.get("north")), Decimal(latlonbox.get("south"))
            lone, lonw = Decimal(latlonbox["east"]), Decimal(latlonbox["west"])
            lat: Decimal = avg(latn, lats)
            lon: Decimal = avg(lone, lonw)
            building_coords.append({"id": building.get("id"), "desc": building.get("desc"), "center-coord": {"lat":str(lat),"lon":str(lon)}, "public-transport":nearby((lat,lon),stations)})

    return building_coords
