import dataframely as dy
import polars as pl
import pytest

from external.schemas.location_images import LocationImagesSchema


def _row(**overrides: object) -> dict[str, object]:
    base = {
        "key": "x",
        "name": "x.webp",
        "author_url": None,
        "author_text": "X",
        "source_url": None,
        "source_text": None,
        "license_url": None,
        "license_text": "CC0",
    }
    base.update(overrides)
    return base


def test_location_images_schema_rejects_empty_key() -> None:
    """Empty key fails the `key_non_empty` rule."""
    invalid = pl.DataFrame([_row(key="")], schema=LocationImagesSchema.to_polars_schema())
    with pytest.raises(dy.exc.ValidationError):
        LocationImagesSchema.validate(invalid)


def test_location_images_schema_accepts_all_nullable_fields() -> None:
    """All metadata columns must be nullable to match mat-view semantics."""
    valid = pl.DataFrame(
        [
            _row(
                name=None,
                author_url=None,
                author_text=None,
                source_url=None,
                source_text=None,
                license_url=None,
                license_text=None,
            )
        ],
        schema=LocationImagesSchema.to_polars_schema(),
    )
    LocationImagesSchema.validate(valid)
