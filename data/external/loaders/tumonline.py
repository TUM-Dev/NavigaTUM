import typing

import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.tumonline import BuildingsSchema, OrgsSchema, UsagesSchema


def load_usages() -> pl.DataFrame:
    """Load the TUMonline usage catalogue. Dtypes are enforced by `UsagesSchema`."""
    return pl.read_csv(
        RESULTS_PATH / "usages_tumonline.csv",
        schema=UsagesSchema.to_polars_schema(),
    )


def load_orgs(lang: typing.Literal["de", "en"]) -> pl.DataFrame:
    """Load the TUMonline organisation catalogue for `lang`. Dtypes enforced by `OrgsSchema`."""
    return pl.read_csv(
        RESULTS_PATH / f"orgs-{lang}_tumonline.csv",
        schema=OrgsSchema.to_polars_schema(),
    )


def load_buildings() -> pl.DataFrame:
    """Load the TUMonline building catalogue. Dtypes enforced by `BuildingsSchema`."""
    return pl.read_csv(
        RESULTS_PATH / "buildings_tumonline.csv",
        schema=BuildingsSchema.to_polars_schema(),
    )
