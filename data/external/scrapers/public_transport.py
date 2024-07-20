import csv
import json
import logging
from zipfile import ZipFile

from external.scraping_utils import _download_file, CACHE_PATH
from utils import setup_logging

MVV_OPENDATA_URL = "https://www.mvv-muenchen.de/fileadmin/mediapool/02-Fahrplanauskunft/03-Downloads/openData"
MVV_GTFS_URL = f"{MVV_OPENDATA_URL}/gtfs_mvv_ohneShape_01.zip"
MVV_HST_REPORT_URL = f"{MVV_OPENDATA_URL}/MVV_HSTReport2312.csv"  # train/tram stations + some bus stations
PUBLIC_TRANSPORT_CACHE_PATH = CACHE_PATH / "public_transport"


def _load_bus_stations(stations: dict) -> None:
    """Load the bus stations from the MVV GTFS data and add them to stations dict"""
    _download_file(MVV_GTFS_URL, PUBLIC_TRANSPORT_CACHE_PATH / "fahrplandaten.zip")
    with ZipFile(PUBLIC_TRANSPORT_CACHE_PATH / "fahrplandaten.zip") as file_zip:
        file_zip.extract("stops.txt", PUBLIC_TRANSPORT_CACHE_PATH)
    with open(PUBLIC_TRANSPORT_CACHE_PATH / "stops.txt", encoding="utf-8") as file:
        lines = list(csv.DictReader(file, delimiter=","))
    repeat_later = []  # when parent station is not already in dict
    for line in lines:
        if line["location_type"]:
            stations.setdefault(
                line["stop_id"],
                {
                    "station_id": line["stop_id"],
                    "name": line["stop_name"],
                    "lat": float(line["stop_lat"]),
                    "lon": float(line["stop_lon"]),
                    "sub_stations": [],
                },
            )
        else:
            sub_station = {
                "station_id": line["stop_id"],
                "name": line["stop_name"],
                "lat": float(line["stop_lat"]),
                "lon": float(line["stop_lon"]),
                "parent": line["parent_station"],
            }
            if not sub_station["parent"]:
                sub_station["parent"] = ":".join(line["stop_id"].split(":")[:3])

            if parent := stations.get(line["parent_station"]):
                parent["sub_stations"].append(sub_station)
            else:
                repeat_later.append(sub_station)

    for sub in repeat_later:
        if parent := stations.get(sub["parent"]):
            parent["sub_stations"].append(sub)
        elif sub["station_id"]:
            logging.warning(f"{sub['name']} with id {sub['station_id']} has no parent in our data")


def _load_train_stations(stations: dict) -> None:
    """Load the bus stations from the MVV_HST_REPORT data and add them to stations dict"""
    _download_file(MVV_HST_REPORT_URL, PUBLIC_TRANSPORT_CACHE_PATH / "train_stations.csv")
    with open(PUBLIC_TRANSPORT_CACHE_PATH / "train_stations.csv", encoding="utf-8") as file:
        lines = list(csv.DictReader(file, delimiter=";"))
    repeat_later = []  # when parent station is not already in dict
    ignored_stations_no_id = 0
    ignored_stations_no_coordinate = 0
    for line in lines:
        if not line["WGS84 X"] or not line["WGS84 X"]:
            ignored_stations_no_coordinate += 1
            continue
        if not line["Globale ID"]:
            ignored_stations_no_id += 1
            continue
        if line["Globale ID"].count(":") == 2:  # example: de:09184:460
            stations.setdefault(
                line["Globale ID"],
                {
                    "station_id": line["Globale ID"],
                    "name": line["Name ohne Ort"],
                    "lat": float(line["WGS84 X"].replace(",", ".")),
                    "lon": float(line["WGS84 Y"].replace(",", ".")),
                    "sub_stations": [],
                },
            )
        else:
            parent_id = ":".join(line["Globale ID"].split(":")[:3])
            sub_station = {
                "station_id": line["Globale ID"],
                "name": line["Name ohne Ort"],
                "lat": float(line["WGS84 X"].replace(",", ".")),
                "lon": float(line["WGS84 Y"].replace(",", ".")),
                "parent": parent_id,
            }

            if parent := stations.get(parent_id):
                parent["sub_stations"].append(sub_station)
            else:
                repeat_later.append(sub_station)
    if ignored_stations_no_id:
        logging.warning(f"ignored because of not having an id: {ignored_stations_no_id}")
    if ignored_stations_no_coordinate:
        logging.warning(f"ignored because of not having coordinates: {ignored_stations_no_coordinate}")
    for sub in repeat_later:
        if parent := stations.get(sub["parent"]):
            parent["sub_stations"].append(sub)
        elif sub["station_id"]:
            logging.warning(f"{sub['name']} with id {sub['station_id']} has no parent in our data")


def scrape_stations() -> None:
    """Scrape the stations from the MVV GTFS data and return them as a list of dicts"""
    logging.info("Scraping the bus and train stations of the MVV")
    stations = {}
    _load_train_stations(stations)
    _load_bus_stations(stations)
    # remove parent property from sub stations
    for station in stations.values():
        for sub in station["sub_stations"]:
            del sub["parent"]
    stations = sorted(stations.values(), key=lambda x: x["lat"])
    with open(CACHE_PATH / "public_transport.json", "w", encoding="utf-8") as file:
        json.dump(stations, file, indent=2, sort_keys=True)


if __name__ == "__main__":
    setup_logging(level=logging.INFO)
    PUBLIC_TRANSPORT_CACHE_PATH.mkdir(exist_ok=True)
    scrape_stations()
