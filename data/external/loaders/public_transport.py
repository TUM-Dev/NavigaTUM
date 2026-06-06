import polars as pl

from external.models.common import RESULTS_PATH
from external.schemas.public_transport import StationsSchema


def load_stations() -> pl.DataFrame:
    """Load the public-transport station catalogue. Dtypes enforced by `StationsSchema`."""
    return StationsSchema.read_parquet(RESULTS_PATH / "public_transport.parquet")
