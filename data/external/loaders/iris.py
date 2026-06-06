import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.iris import IrisRoomsSchema


def load_iris_rooms() -> pl.DataFrame:
    """
    Load the AStA Iris learning-room roster. Dtypes enforced by `IrisRoomsSchema`.

    Both columns stay `String` so building ids like "0201" keep their leading zeros.
    """
    return pl.read_csv(RESULTS_PATH / "iris.csv", schema=IrisRoomsSchema.to_polars_schema())
