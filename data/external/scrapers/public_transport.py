from pathlib import Path
from external.scraping_utils import cached_json
from external.scrapers.roomfinder import scrape_maps
import csv
from decimal import Decimal
from processors.public_transport import nearby
# CSV indexes
STATIONID = 0
NAME = 1
ORT = 2
GLOBAL_ID = 3
WGS84X = 4  # lat
WSG84Y = 6  # lon

def avg(x1, x2):
    return (x1 + x2) / 2

@cached_json("public_transport.json")
def scrape_stations():
    with Path("scrapers/MVV_HSTReport2212.csv").open("r") as file:
        lines = csv.reader(file, delimiter=";")  
        next(lines) # skip first line as it contains row names
        lines = filter(lambda l: not all(not bool(i) for i in l), lines)  # filter out lines where each value is ''
        stations = [{
                    "id":line[STATIONID],
                    "name":line[NAME],
                    "ort":line[ORT],
                    "global-id":line[GLOBAL_ID],
                    "lat":Decimal(line[WGS84X].replace(",", ".")),
                    "lon":Decimal(line[WSG84Y].replace(",", ".")),
                } for line in lines]
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
