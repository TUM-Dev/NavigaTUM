import json
from dataclasses import asdict
from math import acos, cos, radians, sin

from external.models.public_transport import Station

MAXDISTANCE = 1000
METERS_PER_LATITUDE_DEGREE = 111210
MAXDEGDIFF_PER_LATITUDE_DEGREE = MAXDISTANCE / METERS_PER_LATITUDE_DEGREE
EARTH_RADIUS_METERS = 6_371_000


def _distance_via_great_circle(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
    """
    Calculate the approximate distance in meters betweeen two points using the great circle approach
    Basic idea from https://blog.petehouston.com/calculate-distance-of-two-locations-on-earth/
    """
    lat1, lon1, lat2, lon2 = map(radians, [lat1, lon1, lat2, lon2])
    # angular distance using the https://wikipedia.org/wiki/Haversine_formula
    angular_distance = acos(sin(lat1) * sin(lat2) + cos(lat1) * cos(lat2) * cos(lon1 - lon2))
    return EARTH_RADIUS_METERS * angular_distance


def _filter_by_latitude(lat: float, stations: list[Station]) -> list[Station]:
    """
    Returns a list of stations with a latitude within MAXDEGDIFF_PER_LATITUDE_DEGREE of the given latitude.
    Prefiltering the stations by latitude reduces the number of stations to check for distance.
    """
    max_lat = lat + MAXDEGDIFF_PER_LATITUDE_DEGREE
    min_lat = lat - MAXDEGDIFF_PER_LATITUDE_DEGREE
    return [station for station in stations if min_lat < station.lat < max_lat]


def nearby_stations(lat: float, lon: float, stations: list[Station]) -> list[tuple[float, Station]]:
    """returns a list of tuples in form: [distance in meter, station]"""
    results = []
    for station in _filter_by_latitude(lat, stations):
        if (distance := _distance_via_great_circle(station.lat, station.lon, lat, lon)) <= MAXDISTANCE:
            results.append((distance, asdict(station)))  # cast do dict, as dataclass cant be encoded to json by default
    return sorted(results, key=lambda x: x[0])


def add_nearby_public_transport(data):
    """Add the nearby public transport stations to the data"""
    with open("external/results/public_transport.json", encoding="utf-8") as file:
        stations = [Station(**x) for x in json.load(file)]

    for entry in data.values():
        if coords := entry.get("coords", None):
            options = nearby_stations(coords["lat"], coords["lon"], stations)
            entry["nearby_public_transport"] = options
