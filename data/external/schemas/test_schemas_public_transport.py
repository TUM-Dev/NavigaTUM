import dataframely as dy
import polars as pl
import pytest

from external.loaders.public_transport import load_stations
from external.schemas.public_transport import StationsSchema


def test_committed_stations_parquet_satisfies_schema() -> None:
    """The cached `public_transport.parquet` must satisfy `StationsSchema` (drift gate)."""
    StationsSchema.validate(load_stations())


def test_stations_schema_rejects_missing_column() -> None:
    incomplete = pl.DataFrame({"dhid": ["X"]})
    with pytest.raises(dy.exc.SchemaError):
        StationsSchema.validate(incomplete)
