import json
from dataclasses import asdict

from external.models.public_transport import Station
from utils import distance_via_great_circle

MAXDISTANCE = 1000
METERS_PER_LATITUDE_DEGREE = 111210
MAXDEGDIFF_PER_LATITUDE_DEGREE = MAXDISTANCE / METERS_PER_LATITUDE_DEGREE


def _filter_by_latitude(lat: float, stations: list[Station]) -> list[Station]:
    """
    Returns a list of stations with a latitude within MAXDEGDIFF_PER_LATITUDE_DEGREE of the given latitude.
    Prefiltering the stations by latitude reduces the number of stations to check for distance.
    """
    max_lat = lat + MAXDEGDIFF_PER_LATITUDE_DEGREE
    min_lat = lat - MAXDEGDIFF_PER_LATITUDE_DEGREE
    return [station for station in stations if min_lat < station.lat < max_lat]


def nearby_stations(lat: float, lon: float, stations: list[Station]) -> list[dict]:
    """returns a list of tuples in form: [distance in meter, station]"""
    results = []
    for station in _filter_by_latitude(lat, stations):
        if (distance := distance_via_great_circle(station.lat, station.lon, lat, lon)) <= MAXDISTANCE:
            station_dict = {"distance": distance} | asdict(station)
            results.append(station_dict)
    return sorted(results, key=lambda x: x["distance"])


def add_nearby_public_transport(data):
    """Add the nearby public transport stations to the data"""
    stations = Station.load_all()

    for entry in data.values():
        if coords := entry.get("coords", None):  # noqa: SIM102
            if nearby_mvg := nearby_stations(coords["lat"], coords["lon"], stations):
                poi = entry.get("poi", {})
                poi["nearby_public_transport"] = {"mvg": [nearby_mvg]}
                entry["poi"] = poi
