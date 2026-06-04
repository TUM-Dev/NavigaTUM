import dataframely as dy
import polars as pl
import pytest

from external.schemas.ranking_factors import RankingFactorsSchema


def test_ranking_factors_schema_rejects_empty_id() -> None:
    """`RankingFactorsSchema` must reject rows with an empty `id`."""
    invalid = pl.DataFrame(
        {
            "id": [""],
            "rank_type": [1],
            "rank_combined": [1],
            "rank_usage": [1],
            "rank_custom": [1],
            "rank_boost": [1],
        },
        schema=RankingFactorsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        RankingFactorsSchema.validate(invalid)


def test_ranking_factors_schema_rejects_missing_column() -> None:
    """`RankingFactorsSchema` must reject a frame missing required columns."""
    incomplete = pl.DataFrame({"id": ["x"]})
    with pytest.raises(dy.exc.SchemaError):
        RankingFactorsSchema.validate(incomplete)


def test_ranking_factors_schema_accepts_null_rank_columns() -> None:
    """`rank_*` columns must be nullable to match the mat-view semantics."""
    valid = pl.DataFrame(
        {
            "id": ["x"],
            "rank_type": [None],
            "rank_combined": [None],
            "rank_usage": [None],
            "rank_custom": [None],
            "rank_boost": [None],
        },
        schema=RankingFactorsSchema.to_polars_schema(),
    )
    RankingFactorsSchema.validate(valid)


def test_ranking_factors_schema_rejects_duplicate_id() -> None:
    """`id` is the primary key; duplicates must fail validation."""
    duplicates = pl.DataFrame(
        {
            "id": ["x", "x"],
            "rank_type": [1, 2],
            "rank_combined": [1, 2],
            "rank_usage": [1, 2],
            "rank_custom": [1, 2],
            "rank_boost": [1, 2],
        },
        schema=RankingFactorsSchema.to_polars_schema(),
    )
    with pytest.raises(dy.exc.ValidationError):
        RankingFactorsSchema.validate(duplicates)
