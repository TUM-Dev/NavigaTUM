from decimal import Decimal
import math
from math import radians, sin, cos, acos

MAXDISTANCE = 1000  # max distance from building
MAXDEGDIFF_LAT=MAXDISTANCE*(1/111210) #111210 is the lenght of 1° lat in meters

# from https://medium.com/@petehouston/calculate-distance-of-two-locations-on-earth-using-python-1501b1944d97
def _great_circle(lon1, lat1, lon2, lat2):
    lon1, lat1, lon2, lat2 = map(radians, [lon1, lat1, lon2, lat2])
    return 6371 * (
        acos(sin(lat1) * sin(lat2) + cos(lat1) * cos(lat2) * cos(lon1 - lon2))
    )

def _lat_bin_search(position,lst:list[dict])->list[dict]:
  """returns a sublist of stations that are within MAXDISTANCE, assumes the every point has the same lon value."""
  lower,upper=0,len(lst)
  current=math.ceil((upper-lower)/2)
  while not abs((distance:=Decimal(lst[current]["lat"])-Decimal(position[0]))) <= MAXDEGDIFF_LAT and not lower==upper:
    if distance>0:
       upper=current
    else:
      lower=current
    current=lower+math.ceil((upper-lower)/2)
  interval= _get_lat_interval(position,lst,current)
  return lst[interval[0]:interval[1]+1]
  
def _get_lat_interval(position:tuple, lst:list[dict], point_in_interval:int)->tuple[int,int]: #TODO invalid values
  upperbound=lowerbound=point_in_interval
  while upperbound+5<=len(lst)-1 and Decimal(lst[upperbound+5]["lat"])-Decimal(position[0]) <= MAXDEGDIFF_LAT:
    upperbound+=5
  while upperbound+1<=len(lst)-1 and Decimal(lst[upperbound+1]["lat"])-Decimal(position[0]) <= MAXDEGDIFF_LAT:
    upperbound+=1
  while lowerbound-5>=0 and abs(Decimal(lst[lowerbound-5]["lat"])-Decimal(position[0])) <= MAXDEGDIFF_LAT:
    lowerbound-=5
  if lowerbound-1>=0 and abs(Decimal(lst[lowerbound-1]["lat"])-Decimal(position[0])) <= MAXDEGDIFF_LAT:
    lowerbound-=1
  return (lowerbound,upperbound)


def nearby(building_coords:tuple, stations: list[dict]) -> list[tuple[float,dict]]:
  """returns a list of tuples in form: [distance in meter, station]"""
  results=[]
  for station in _lat_bin_search(building_coords,stations):
    if (distance:=(_great_circle(float(station["lat"]),float(station["lon"]),building_coords[0],building_coords[1]))*1000) <=MAXDISTANCE:
      results.append((distance,station))
  return sorted(results,key=lambda x: x[0])

if __name__=="__main__":
    import json
    with open("public_transport.json") as file:
        stations=json.load(file)
        print(nearby((48.1488320687913,11.4606435143223),stations))

