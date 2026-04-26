import dataframely as dy
import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.roomfinder import BuildingsSchema, MapsSchema, RoomsSchema


def _read_csv_typed(path, schema: type[dy.Schema]) -> pl.DataFrame:
    """Read a CSV against a dataframely schema, with whitespace stripping.

    Narrowed dtypes (`pl.Enum`, `pl.Categorical`) are read as `pl.String` first so we can
    `str.strip_chars()` before casting to the final dtype. This both mimics the old Pydantic
    `str_strip_whitespace=True` behaviour and avoids polluting Categorical dictionaries — and
    avoids Enum cast errors on stray whitespace (e.g. `"Nationalpark Berchtesgaden\\n"`).
    """
    full = schema.to_polars_schema()
    narrowed = {k: v for k, v in full.items() if isinstance(v, (pl.Enum, pl.Categorical))}
    read_schema = pl.Schema({k: (pl.String if k in narrowed else v) for k, v in full.items()})

    df = pl.read_csv(path, schema=read_schema)
    str_cols = [c for c, d in df.schema.items() if d == pl.String]
    df = df.with_columns(pl.col(c).str.strip_chars() for c in str_cols)
    if narrowed:
        df = df.with_columns(pl.col(k).cast(v) for k, v in narrowed.items())
    return df


def load_buildings() -> pl.DataFrame:
    """Load the Roomfinder building catalogue. Dtypes enforced by `BuildingsSchema`."""
    return _read_csv_typed(RESULTS_PATH / "buildings_roomfinder.csv", BuildingsSchema)


def load_rooms() -> pl.DataFrame:
    """Load the Roomfinder room catalogue. Dtypes enforced by `RoomsSchema`."""
    return _read_csv_typed(RESULTS_PATH / "rooms_roomfinder.csv", RoomsSchema)


def load_maps() -> pl.DataFrame:
    """Load the Roomfinder map catalogue. Dtypes enforced by `MapsSchema`."""
    return _read_csv_typed(RESULTS_PATH / "maps_roomfinder.csv", MapsSchema)
