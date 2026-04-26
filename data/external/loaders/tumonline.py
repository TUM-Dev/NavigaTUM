import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.tumonline import UsagesSchema


def load_usages() -> pl.DataFrame:
    """Load the TUMonline usage catalogue. Dtypes are enforced by `UsagesSchema`."""
    return pl.read_csv(
        RESULTS_PATH / "usages_tumonline.csv",
        schema_overrides=UsagesSchema.to_polars_schema(),
    )
