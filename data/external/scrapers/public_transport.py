import csv
from zipfile import ZipFile

from external.scraping_utils import _download_file, CACHE_PATH, cached_json

MVV_GTFS_URL = "https://www.mvv-muenchen.de/fileadmin/mediapool/02-Fahrplanauskunft/03-Downloads/openData/mvv_gtfs.zip"
MVV_HST_REPORT_URL = "https://www.mvv-muenchen.de/fileadmin/mediapool/02-Fahrplanauskunft/03-Downloads/openData/MVV_HSTReport2212.csv"  # train/tram stations + some bus stations
PUBLIC_TRANSPORT_CACHE_PATH = CACHE_PATH / "public_transport"


def _load_bus_stations() -> dict:
    with open(PUBLIC_TRANSPORT_CACHE_PATH / "stops.txt", encoding="utf-8") as file:
        lines = csv.DictReader(file, delimiter=",")
        stations = {}
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

                if parent := stations.get(line["parent_station"]):
                    parent["sub_stations"].append(sub_station)
                else:
                    repeat_later.append(sub_station)

        for sub in repeat_later:
            if parent := stations.get(sub["parent"]):
                parent["sub_stations"].append(sub)
        return stations


def _load_train_stations(stations: dict) -> list:
    with open(PUBLIC_TRANSPORT_CACHE_PATH / "train_stations.csv", encoding="utf-8") as file:
        lines = csv.DictReader(file, delimiter=";")
        lines = list(filter(lambda x: x["\ufeffHstNummer"], lines))
        repeat_later = []  # when parent station is not already in dict
        for line in lines:
            if (
                len(line["Globale ID"].split(":")) == 3
            ):  # parentstation ex. de:09184:460 substation ex. de:09184:460:30:1
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
                parent_id = ".".join(line["Globale ID"].split(":")[0:3])
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
            for sub in repeat_later:
                if parent := stations.get(sub["parent"]):
                    parent["sub_stations"].append(sub)
        return stations.values()


@cached_json("public_transport.json")
def scrape_stations():
    """Scrape the stations from the MVV GTFS data and return them as a list of dicts"""
    _download_file(MVV_GTFS_URL, PUBLIC_TRANSPORT_CACHE_PATH / "fahrplandaten.zip")
    with ZipFile(PUBLIC_TRANSPORT_CACHE_PATH / "fahrplandaten.zip") as file_zip:
        file_zip.extract("stops.txt", PUBLIC_TRANSPORT_CACHE_PATH)
    _download_file(MVV_HST_REPORT_URL, PUBLIC_TRANSPORT_CACHE_PATH / "train_stations.csv")
    stations = _load_bus_stations()
    stations = _load_train_stations(stations)
    # remove parent property from sub stations
    for station in stations:
        for sub in station["sub_stations"]:
            del sub["parent"]
    return sorted(stations, key=lambda x: x["lat"])
