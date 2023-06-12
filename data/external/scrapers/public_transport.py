from mvg_api import get_nearby_stations
from external.scraping_utils import cached_json
from roomfinder import scrape_maps

@cached_json("public_transport.json")
def scrape_stations():
    buildings=scrape_maps()
    for building in buildings:
        lat=building.latlonbox.north
        lon=building.latlonbox.east
        print(get_nearby_stations(lat,lon))