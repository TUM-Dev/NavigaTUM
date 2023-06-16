from dataclasses import dataclass
from pathlib import Path
from typing import List
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


@dataclass(frozen=True)
class Coords:
    lat: Decimal
    lon: Decimal


@dataclass
class Station:
    id: int
    Name: str
    Ort: str
    global_id: str
    lat: Decimal
    lon: Decimal


def nearby(building: Coords, stations: List[Station]) -> List[Station]:
    return list(
        filter(
            lambda station: distance((station.lat, station.lon), (building.lat, building.lon)).km <= MAXDISTANCE,
            stations,
        )
    )


# @cached_json("public_transport.json")
def scrape_stations():
    with Path("data/external/scrapers/MVV_HSTReport2212.csv").open("r") as file:
        lines = list(csv.reader(file, delimiter=";"))[1:]  # ignore first line as it contains row names
        lines = filter(lambda l: not all(not bool(i) for i in l), lines)  # filter out lines where each value is ''
    stations = list(
        map(
            lambda station: Station(
                station[STATIONID],
                station[NAME],
                station[ORT],
                station[GLOBAL_ID],
                Decimal(station[WGS84X].replace(",", ".")),
                Decimal(station[WSG84Y].replace(",", ".")),
            ),
            lines,
        )
    )
    buildings = scrape_maps()
    building_coords: list = []
    for building in buildings:
        # ignore entries that dont have assigned lon/lat
        if latlonbox := building.get("latlonbox"):
            # since lat and lon have rather long decimal points it is probably best to not use floats
            latn, lats = Decimal(latlonbox["north"]), Decimal(latlonbox["south"])
            lone, lonw = Decimal(latlonbox["east"]), Decimal(latlonbox["west"])
            lat: Decimal = avg(latn, lats)
            lon: Decimal = avg(lone, lonw)
            coords = Coords(lat, lon)
            building_coords.append({"id": building.get("id"), "desc": building.get("desc"), "center-coord": coords})
            print(nearby(coords, stations))
            print(building)
            print(coords)

    # print(building_coords)
