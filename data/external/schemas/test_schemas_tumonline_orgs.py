import dataframely as dy
import polars as pl
import pytest

from external.loaders.tumonline_orgs import load_tumonline_orgs
from external.schemas._drift_gate import assert_satisfies_schema
from external.schemas.tumonline_orgs import TumonlineOrgsSchema


def test_committed_orgs_csvs_merge_into_schema_compliant_frame() -> None:
    """The merged TUMonline-orgs frame must satisfy `TumonlineOrgsSchema` (drift gate)."""
    assert_satisfies_schema(TumonlineOrgsSchema, load_tumonline_orgs())


def test_orgs_schema_rejects_non_positive_id() -> None:
    """`TumonlineOrgsSchema` must reject rows with a non-positive `org_id`."""
    invalid = pl.DataFrame(
        {
            "org_id": [0],
            "code": ["X"],
            "name_de": ["X"],
            "name_en": ["X"],
            "path_de": [None],
            "path_en": [None],
        },
        schema=TumonlineOrgsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        TumonlineOrgsSchema.validate(invalid)


def test_orgs_schema_rejects_missing_column() -> None:
    """`TumonlineOrgsSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"org_id": [1]})
    with pytest.raises(dy.exc.SchemaError):
        TumonlineOrgsSchema.validate(incomplete)
