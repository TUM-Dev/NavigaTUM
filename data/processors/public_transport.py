from decimal import Decimal
import math
from math import radians, sin, cos, acos

from data.external.models.public_transport import Station


MAXDISTANCE=1000
METERS_PER_LATITUDE_DEGREE = 111210
MAXDEGDIFF_PER_LATITUDE_DEGREE = MAXDISTANCE / METERS_PER_LATITUDE_DEGREE
EARTH_RADIUS_METERS = 6_371_000

# from https://medium.com/@petehouston/calculate-distance-of-two-locations-on-earth-using-python-1501b1944d97
def _great_circle(lon1:float, lat1:float, lon2:float, lat2:float)->float:
    """Calculate the approximate distance in meters betweeen two points using the great circle approach"""
    lon1, lat1, lon2, lat2 = map(radians, [lon1, lat1, lon2, lat2])
    # angular distance using the https://wikipedia.org/wiki/Haversine_formula
    angular_distance = acos(sin(lat1) * sin(lat2) + cos(lat1) * cos(lat2) * cos(lon1 - lon2))
    return EARTH_RADIUS_METERS * angular_distance * 1000


def _lat_bin_search(position,lst:list[Station])->list[Station]:
  """do binary search until a point is found, that is within MAXDISTANCE to position. This treats all longitude values as the same. 
  Then expands in both directions from that point, until the next one would be outside of MAXDISTANCE"""
  lower,upper=0,len(lst)
  current=math.ceil((upper-lower)/2)
  while not abs((distance:=Decimal(lst[current]["lat"])-Decimal(position[0]))) <= MAXDEGDIFF_PER_LATITUDE_DEGREE and not lower==upper:
    if distance>0:
       upper=current
    else:
      lower=current
    current=lower+math.ceil((upper-lower)/2)
  interval= _get_lat_interval(position,lst,current)
  return lst[interval[0]:interval[1]+1]

  
def _get_lat_interval(position:tuple, lst:list[Station], point_in_interval:int)->tuple[int,int]:
  """expanding the interval that of values whose lat values are within MAXDEGDIFF_LAT to position, starting from point_in_interval"""
  upperbound=lowerbound=point_in_interval
  while upperbound+5<=len(lst)-1 and Decimal(lst[upperbound+5]["lat"])-Decimal(position[0]) <= MAXDEGDIFF_PER_LATITUDE_DEGREE:
    upperbound+=5
  while upperbound+1<=len(lst)-1 and Decimal(lst[upperbound+1]["lat"])-Decimal(position[0]) <= MAXDEGDIFF_PER_LATITUDE_DEGREE:
    upperbound+=1
  while lowerbound-5>=0 and abs(Decimal(lst[lowerbound-5]["lat"])-Decimal(position[0])) <= MAXDEGDIFF_PER_LATITUDE_DEGREE:
    lowerbound-=5
  while lowerbound-1>=0 and abs(Decimal(lst[lowerbound-1]["lat"])-Decimal(position[0])) <= MAXDEGDIFF_PER_LATITUDE_DEGREE:
    lowerbound-=1
  return (lowerbound,upperbound)



def _lat_search(lat:float, stations:list[Station]):
    max_lat = lat[0] + MAXDEGDIFF_PER_LATITUDE_DEGREE
    min_lat = lat[0] - MAXDEGDIFF_PER_LATITUDE_DEGREE
    return [station for station in stations if min_lat < float(station["lat"]) < max_lat]

def nearby(building_coords:tuple, stations: list[Station]) -> list[tuple[float,Station]]:
  """returns a list of tuples in form: [distance in meter, station]"""
  results=[]
  for station in _lat_bin_search(building_coords,stations):
    if (distance:=_great_circle(float(station["lat"]),float(station["lon"]),building_coords[0],building_coords[1])) <=MAXDISTANCE:
      results.append((distance,station))
  return sorted(results,key=lambda x: x[0])
