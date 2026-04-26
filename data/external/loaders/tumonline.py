import typing
from pathlib import Path

import dataframely as dy
import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.tumonline import BuildingsSchema, OrgsSchema, RoomsSchema, UsagesSchema


def _read_csv_typed(path: Path, schema: type[dy.Schema]) -> pl.DataFrame:
    """
    Read a CSV against a dataframely schema, with whitespace stripping.

    Narrowed dtypes (`pl.Enum`, `pl.Categorical`) are read as `pl.String` first so we can
    `str.strip_chars()` before casting to the final dtype. This both mimics the old Pydantic
    `str_strip_whitespace=True` behaviour and avoids polluting Categorical dictionaries with
    whitespace-padded duplicates.
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


def load_usages() -> pl.DataFrame:
    """Load the TUMonline usage catalogue. Dtypes enforced by `UsagesSchema`."""
    return _read_csv_typed(RESULTS_PATH / "usages_tumonline.csv", UsagesSchema)


def load_orgs(lang: typing.Literal["de", "en"]) -> pl.DataFrame:
    """Load the TUMonline organisation catalogue for `lang`. Dtypes enforced by `OrgsSchema`."""
    return _read_csv_typed(RESULTS_PATH / f"orgs-{lang}_tumonline.csv", OrgsSchema)


def load_buildings() -> pl.DataFrame:
    """Load the TUMonline building catalogue. Dtypes enforced by `BuildingsSchema`."""
    return _read_csv_typed(RESULTS_PATH / "buildings_tumonline.csv", BuildingsSchema)


def load_rooms() -> pl.DataFrame:
    """Load the TUMonline room catalogue. Dtypes enforced by `RoomsSchema`."""
    return _read_csv_typed(RESULTS_PATH / "rooms_tumonline.csv", RoomsSchema)
