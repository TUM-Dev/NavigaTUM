import dataframely as dy
import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.roomfinder import BuildingsSchema, MapsSchema, RoomsSchema


def _strip_whitespace(df: pl.DataFrame, schema: type[dy.Schema]) -> pl.DataFrame:
    """Strip leading/trailing whitespace on every String column (matches old Pydantic behaviour)."""
    str_cols = [name for name, dtype in schema.to_polars_schema().items() if dtype == pl.String]
    return df.with_columns(pl.col(c).str.strip_chars() for c in str_cols)


def load_buildings() -> pl.DataFrame:
    """Load the Roomfinder building catalogue. Dtypes enforced by `BuildingsSchema`."""
    return _strip_whitespace(
        pl.read_csv(RESULTS_PATH / "buildings_roomfinder.csv", schema=BuildingsSchema.to_polars_schema()),
        BuildingsSchema,
    )


def load_rooms() -> pl.DataFrame:
    """Load the Roomfinder room catalogue. Dtypes enforced by `RoomsSchema`."""
    return _strip_whitespace(
        pl.read_csv(RESULTS_PATH / "rooms_roomfinder.csv", schema=RoomsSchema.to_polars_schema()),
        RoomsSchema,
    )


def load_maps() -> pl.DataFrame:
    """Load the Roomfinder map catalogue. Dtypes enforced by `MapsSchema`."""
    return _strip_whitespace(
        pl.read_csv(RESULTS_PATH / "maps_roomfinder.csv", schema=MapsSchema.to_polars_schema()),
        MapsSchema,
    )
