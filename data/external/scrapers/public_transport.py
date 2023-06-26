from pathlib import Path
from external.scraping_utils import cached_json
import csv
# CSV indexes
STATIONID = "stop_id"
NAME = "stop_name"
TYPE= "location_type"
LATITUDE = "stop_lat" 
LONGITUDE = "stop_lon"
PARENT="parent_station"


@cached_json("public_transport.json")
def scrape_stations():
    with Path("scrapers/stops.txt").open("r") as file: #the file is from the zip file "Soll-Fahrplandaten" from https://www.mvv-muenchen.de/fahrplanauskunft/fuer-entwickler/opendata/index.html
        lines = csv.DictReader(file, delimiter=",")  
        stations={}
        repeat_later=[] #when parent station is not already in dict
        for line in lines:
            if line[TYPE]:
                stations.setdefault(line[STATIONID],{
                    "id":line[STATIONID],
                    "name":line[NAME],
                    "lat":line[LATITUDE].replace(",", "."),
                    "lon":line[LONGITUDE].replace(",", "."),
                    "sub_stations":[]
                } )
            else:
                sub_station={
                        "id":line[STATIONID],
                        "name":line[NAME],
                        "lat":line[LATITUDE].replace(",", "."),
                        "lon":line[LONGITUDE].replace(",", "."),
                        "parent":line[PARENT]
                    }
                
                if (parent:=stations.get(line[PARENT])):
                    parent["sub_stations"].append(sub_station)
                else:
                    repeat_later.append(sub_station)

        for sub in repeat_later:
            if (parent:=stations.get(sub["parent"])):
                parent["sub_stations"].append(sub)
        return sorted(stations.values(),key=lambda x: x["lat"])