import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.roomfinder import BuildingsSchema, MapsSchema, RoomsSchema


def load_buildings() -> pl.DataFrame:
    """Load the Roomfinder building catalogue. Dtypes enforced by `BuildingsSchema`."""
    return pl.read_csv(
        RESULTS_PATH / "buildings_roomfinder.csv",
        schema=BuildingsSchema.to_polars_schema(),
    )


def load_rooms() -> pl.DataFrame:
    """Load the Roomfinder room catalogue. Dtypes enforced by `RoomsSchema`."""
    return pl.read_csv(
        RESULTS_PATH / "rooms_roomfinder.csv",
        schema=RoomsSchema.to_polars_schema(),
    )


def load_maps() -> pl.DataFrame:
    """Load the Roomfinder map catalogue. Dtypes enforced by `MapsSchema`."""
    return pl.read_csv(
        RESULTS_PATH / "maps_roomfinder.csv",
        schema=MapsSchema.to_polars_schema(),
    )
