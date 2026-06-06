import logging

import polars as pl
import requests
from utils import setup_logging

from external.schemas.iris import IrisRoomsSchema
from external.scraping_utils import CACHE_PATH

IRIS_API_URL = "https://iris.asta.tum.de/api/"

_logger = logging.getLogger(__name__)


def scrape_iris() -> None:
    """
    Write `iris.csv` from the live AStA Iris endpoint.

    Refuses to overwrite with an empty roster, so a transient AStA outage leaves the previously
    scraped data in place rather than wiping coverage.
    """
    response = requests.get(IRIS_API_URL, timeout=30)
    response.raise_for_status()
    rooms = response.json()["raeume"]
    if not rooms:
        raise RuntimeError("Iris returned no rooms - refusing to overwrite the roster")

    rows = [
        {"raum_nr_architekt": room["raum_nr_architekt"], "gebaeude_code": str(room["gebaeude_code"])} for room in rooms
    ]
    df = pl.DataFrame(rows, schema=IrisRoomsSchema.to_polars_schema()).sort("raum_nr_architekt")
    df.write_csv(CACHE_PATH / "iris.csv")
    _logger.info(f"Scraped {df.height} Iris rooms across {df['gebaeude_code'].n_unique()} buildings")


if __name__ == "__main__":
    setup_logging()
    CACHE_PATH.mkdir(exist_ok=True)
    scrape_iris()
