import pytest

from processors.images import parse_image_filename


@pytest.mark.parametrize(
    ("image_name", "expected"),
    [
        ("mi_0.webp", ("mi", 0)),
        ("5101.EG.917_2.webp", ("5101.EG.917", 2)),
        ("9d02ddd940c43f87_0.webp", ("9d02ddd940c43f87", 0)),
        # Submitted-event images carry an underscore in the id; only the trailing index splits off.
        ("event_9d02ddd940c43f87_0.webp", ("event_9d02ddd940c43f87", 0)),
    ],
)
def test_parse_image_filename(image_name: str, expected: tuple[str, int]) -> None:
    """Only the trailing `_<index>` splits off, so ids may contain underscores."""
    assert parse_image_filename(image_name) == expected


@pytest.mark.parametrize("image_name", ["mi_0.png", "no_index.webp", "trailing_.webp"])
def test_parse_image_filename_rejects_malformed(image_name: str) -> None:
    """A non-webp name or a missing integer index is a hard error."""
    with pytest.raises(RuntimeError):
        parse_image_filename(image_name)
