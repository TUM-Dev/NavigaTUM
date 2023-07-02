from pathlib import Path
from external.scraping_utils import cached_json, _download_file,CACHE_PATH
from zipfile import ZipFile
import csv


def _download_zip(filepath):
    url="https://www.mvv-muenchen.de/fileadmin/mediapool/02-Fahrplanauskunft/03-Downloads/openData/mvv_gtfs.zip"
    _download_file(url,filepath)

def _extract_stops(zip_location,target_dir):
    zip=ZipFile(zip_location)
    zip.extract("stops.txt",target_dir)

def _load_bus_stations(parent_dir)->dict:
    # CSV indexes
    STATIONID = "stop_id"
    NAME = "stop_name"
    TYPE= "location_type"
    LATITUDE = "stop_lat" 
    LONGITUDE = "stop_lon"
    PARENT="parent_station"

    with Path(parent_dir / "stops.txt").open("r") as file:
            lines = csv.DictReader(file, delimiter=",")  
            stations={}
            repeat_later=[] #when parent station is not already in dict
            for line in lines:
                if line[TYPE]:
                    stations.setdefault(line[STATIONID],{
                        "id":line[STATIONID],
                        "name":line[NAME],
                        "lat":float(line[LATITUDE]),
                        "lon":float(line[LONGITUDE]),
                        "sub_stations":[]
                    } )
                else:
                    sub_station={
                            "id":line[STATIONID],
                            "name":line[NAME],
                            "lat":float(line[LATITUDE]),
                            "lon":float(line[LONGITUDE]),
                            "parent":line[PARENT]
                        }
                    
                    if (parent:=stations.get(line[PARENT])):
                        parent["sub_stations"].append(sub_station)
                    else:
                        repeat_later.append(sub_station)

            for sub in repeat_later:
                if (parent:=stations.get(sub["parent"])):
                    parent["sub_stations"].append(sub)
            return stations
    
def _load_train_stations(parent_dir,stations:dict):
    STATIONNR="\ufeffHstNummer"
    NAME="Name ohne Ort"
    STATIONID="Globale ID"
    LATITUDE="WGS84 X"
    LONGITUDE="WGS84 Y"
    with Path(parent_dir / "train_stations.csv").open("r") as file:
        lines = csv.DictReader(file, delimiter=";")  
        lines=list(filter(lambda x: x[STATIONNR],lines))
        repeat_later=[] #when parent station is not already in dict
        for line in lines:
            if len(line[STATIONID].split(":"))==3: # parentstation ex. de:09184:460 substation ex. de:09184:460:30:1
                stations.setdefault(line[STATIONID],{
                    "id":line[STATIONID],
                    "name":line[NAME],
                    "lat":float(line[LATITUDE].replace(",",".")),
                    "lon":float(line[LONGITUDE].replace(",",".")),
                    "sub_stations":[]
                } )
            else:
                parent=".".join(line[STATIONID].split(":")[0:3])
                sub_station={
                        "id":line[STATIONID],
                        "name":line[NAME],
                        "lat":float(line[LATITUDE].replace(",",".")),
                        "lon":float(line[LONGITUDE].replace(",",".")),
                        "parent":parent
                    }
                
                if (parent:=stations.get(parent)):
                    parent["sub_stations"].append(sub_station)
                else:
                    repeat_later.append(sub_station)
            for sub in repeat_later:
                if (parent:=stations.get(sub["parent"])):
                    parent["sub_stations"].append(sub)
        return sorted(stations.values(),key=lambda x: x["lat"])

@cached_json("public_transport.json")
def scrape_stations():
    parent_dir=CACHE_PATH / "public_transport"
    _download_zip(parent_dir / "fahrplandaten.zip") #bus stations
    _download_file("https://www.mvv-muenchen.de/fileadmin/mediapool/02-Fahrplanauskunft/03-Downloads/openData/MVV_HSTReport2212.csv", parent_dir / "train_stations.csv") #train/tram stations + some bus stations
    _extract_stops(parent_dir / "fahrplandaten.zip", parent_dir)
    stations=_load_bus_stations(parent_dir)
    return _load_train_stations(parent_dir, stations)