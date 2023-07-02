import csv
from zipfile import ZipFile

from external.scraping_utils import _download_file, CACHE_PATH, cached_json

MVV_GTFS_URL = "https://www.mvv-muenchen.de/fileadmin/mediapool/02-Fahrplanauskunft/03-Downloads/openData/mvv_gtfs.zip"
PUBLIC_TRANSPORT_CACHE_PATH = CACHE_PATH / "public_transport"


@cached_json("public_transport.json")
def scrape_stations():
    """Scrape the stations from the MVV GTFS data and return them as a list of dicts"""
    _download_file(MVV_GTFS_URL, PUBLIC_TRANSPORT_CACHE_PATH / "fahrplandaten.zip")
    with ZipFile(PUBLIC_TRANSPORT_CACHE_PATH / "fahrplandaten.zip") as file_zip:
        file_zip.extract("stops.txt", PUBLIC_TRANSPORT_CACHE_PATH)

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
        # remove parent property from sub stations
        for station in stations.values():
            for sub in station["sub_stations"]:
                del sub["parent"]
        return sorted(stations.values(), key=lambda x: x["lat"])
