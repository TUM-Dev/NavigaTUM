import dataframely as dy
import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.public_transport import StationsSchema


def load_stations() -> dy.DataFrame[StationsSchema]:
    """Load and validate the public-transport station catalogue."""
    return StationsSchema.validate(pl.read_parquet(RESULTS_PATH / "public_transport.parquet"))
