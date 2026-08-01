import dataframely as dy
import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.studierendenwerk import StudierendenwerkSchema

STUDIERENDENWERK_CSV = RESULTS_PATH / "studierendenwerk.csv"


def load_studierendenwerk() -> dy.DataFrame[StudierendenwerkSchema]:
    """Load and validate the cached `studierendenwerk.csv` canteen opening hours."""
    df = pl.read_csv(STUDIERENDENWERK_CSV, schema=pl.Schema(StudierendenwerkSchema))
    return StudierendenwerkSchema.validate(df)
