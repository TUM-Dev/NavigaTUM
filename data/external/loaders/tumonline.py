import typing

import dataframely as dy
import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.tumonline import BuildingsSchema, OrgsSchema, RoomsSchema, UsagesSchema


def _strip_whitespace(df: pl.DataFrame, schema: type[dy.Schema]) -> pl.DataFrame:
    """
    Strip leading/trailing whitespace on every String column.

    Mimics the Pydantic `str_strip_whitespace=True` config the old loaders relied on.
    Some upstream sources (notably TUMonline org names) embed stray tabs/spaces.
    """
    str_cols = [name for name, dtype in schema.to_polars_schema().items() if dtype == pl.String]
    return df.with_columns(pl.col(c).str.strip_chars() for c in str_cols)


def load_usages() -> pl.DataFrame:
    """Load the TUMonline usage catalogue. Dtypes are enforced by `UsagesSchema`."""
    return _strip_whitespace(
        pl.read_csv(RESULTS_PATH / "usages_tumonline.csv", schema=UsagesSchema.to_polars_schema()),
        UsagesSchema,
    )


def load_orgs(lang: typing.Literal["de", "en"]) -> pl.DataFrame:
    """Load the TUMonline organisation catalogue for `lang`. Dtypes enforced by `OrgsSchema`."""
    return _strip_whitespace(
        pl.read_csv(RESULTS_PATH / f"orgs-{lang}_tumonline.csv", schema=OrgsSchema.to_polars_schema()),
        OrgsSchema,
    )


def load_buildings() -> pl.DataFrame:
    """Load the TUMonline building catalogue. Dtypes enforced by `BuildingsSchema`."""
    return _strip_whitespace(
        pl.read_csv(RESULTS_PATH / "buildings_tumonline.csv", schema=BuildingsSchema.to_polars_schema()),
        BuildingsSchema,
    )


def load_rooms() -> pl.DataFrame:
    """Load the TUMonline room catalogue. Dtypes enforced by `RoomsSchema`."""
    return _strip_whitespace(
        pl.read_csv(RESULTS_PATH / "rooms_tumonline.csv", schema=RoomsSchema.to_polars_schema()),
        RoomsSchema,
    )
