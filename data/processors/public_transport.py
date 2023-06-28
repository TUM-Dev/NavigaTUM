from dataclasses import asdict
from math import radians, sin, cos, acos
import json
from data.external.models.public_transport import Station


MAXDISTANCE=1000
METERS_PER_LATITUDE_DEGREE = 111210
MAXDEGDIFF_PER_LATITUDE_DEGREE = MAXDISTANCE / METERS_PER_LATITUDE_DEGREE
EARTH_RADIUS_METERS = 6_371_000

# from https://medium.com/@petehouston/calculate-distance-of-two-locations-on-earth-using-python-1501b1944d97
def _great_circle(lat1:float,lon1:float, lat2:float,lon2:float)->float:
    """Calculate the approximate distance in meters betweeen two points using the great circle approach"""
    lon1, lat1, lon2, lat2 = map(radians, [lon1, lat1, lon2, lat2])
    # angular distance using the https://wikipedia.org/wiki/Haversine_formula
    angular_distance = acos(sin(lat1) * sin(lat2) + cos(lat1) * cos(lat2) * cos(lon1 - lon2))
    return EARTH_RADIUS_METERS * angular_distance


def _lat_search(lat:float, stations:list[Station]):
    max_lat = lat + MAXDEGDIFF_PER_LATITUDE_DEGREE
    min_lat = lat - MAXDEGDIFF_PER_LATITUDE_DEGREE
    return [station for station in stations if min_lat < station.lat < max_lat]


def nearby(building_coords:tuple, stations: list[Station]) -> list[tuple[float,Station]]:
  """returns a list of tuples in form: [distance in meter, station]"""
  results=[]
  for station in _lat_search(building_coords[0],stations):
    if (distance:=_great_circle(station.lat,station.lon,building_coords[0],building_coords[1])) <=MAXDISTANCE:
      results.append((distance,asdict(station))) #cast do dict, as dataclass cant be encoded to json by default
  return sorted(results,key=lambda x: x[0])

def add_nearby_public_transport(data):
  with open('external/results/public_transport.json', 'r') as f:
    stations=[Station(**x) for x in json.load(f)]

  for key,entry in data.items():
    if entry["type"]=="building":
      coords=entry["coords"]
      options=nearby((coords["lat"],coords["lon"]),stations)
      entry["nearby_public_transport"]=options