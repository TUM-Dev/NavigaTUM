import dataframely as dy
import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.ub_tum import UbTumSchema

UB_TUM_CSV = RESULTS_PATH / "ub_tum.csv"


def load_ub_tum() -> dy.DataFrame[UbTumSchema]:
    """Load and validate the cached `ub_tum.csv` UB-TUM branch-library opening hours."""
    df = pl.read_csv(UB_TUM_CSV, schema=UbTumSchema.to_polars_schema())
    return UbTumSchema.validate(df)
