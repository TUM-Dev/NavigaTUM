import logging

import polars as pl
import requests

from external.scraping_utils import CACHE_PATH
from utils import setup_logging

IRIS_API_URL = "https://iris.asta.tum.de/api/"


def scrape_iris() -> None:
    """
    Download the AStA Iris learning-room roster and store it for the build.

    On failure the existing roster is left untouched, so a transient AStA outage degrades to
    the previously-scraped data rather than breaking the build.
    """
    response = requests.get(IRIS_API_URL, timeout=30)
    response.raise_for_status()
    rooms = response.json()["raeume"]

    df = pl.DataFrame(
        {
            "raum_nr_architekt": [room["raum_nr_architekt"] for room in rooms],
            "gebaeude_code": [str(room["gebaeude_code"]) for room in rooms],
        },
        schema={"raum_nr_architekt": pl.String, "gebaeude_code": pl.String},
    )
    df.write_csv(CACHE_PATH / "iris.csv")
    logging.info(f"Scraped {len(df)} Iris rooms across {df['gebaeude_code'].n_unique()} buildings")


if __name__ == "__main__":
    setup_logging()
    scrape_iris()
