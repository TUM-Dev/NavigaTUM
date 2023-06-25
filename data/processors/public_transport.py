from decimal import Decimal
import math
from geopy.distance import great_circle

MAXDISTANCE = 1000  # max distance from building
MAXDEGDIFF_LAT=MAXDISTANCE*(1/111210) #111210 is the lenght of 1° lat in meters

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
  
def _get_lat_interval(position, lst, point_in_interval)->tuple[int,int]: #TODO invalid values
  upperbound=point_in_interval+2
  while Decimal(lst[upperbound+2]["lat"])-Decimal(position[0]) <= MAXDEGDIFF_LAT:
    upperbound+=2
  if Decimal(lst[upperbound+1]["lat"])-Decimal(position[0]) <= MAXDEGDIFF_LAT:
    upperbound+=1
  lowerbound=point_in_interval-2
  while abs(Decimal(lst[lowerbound-2]["lat"])-Decimal(position[0])) <= MAXDEGDIFF_LAT:
    lowerbound-=2
  if abs(Decimal(lst[lowerbound-1]["lat"])-Decimal(position[0])) <= MAXDEGDIFF_LAT:
    lowerbound-=1
  return (lowerbound,upperbound) # stations that are MAXDEGDIFF_LAT away assuming they are on the same lon


def nearby(building_coords:tuple, stations: list[dict]) -> list[dict]:
  results=[]
  for station in _lat_bin_search(building_coords,stations):
    if (distance:=great_circle(building_coords,(float(station["lat"]),float(station["lon"]))).m) <=MAXDISTANCE:
      results.append((distance,station))
  return results

if __name__=="__main__":
    import json
    with open("public_transport.json") as file:
        stations=json.load(file)
        print(len(stations))
        import time
        s=time.time()
        print(nearby((48.1488320687913,11.4606435143223),stations))
        print(time.time()-s)

