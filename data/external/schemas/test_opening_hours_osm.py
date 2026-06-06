import pytest

from external.schemas._opening_hours_osm import assert_osm_parses


@pytest.mark.parametrize(
    "osm",
    [
        "Mo-Fr 08:00-20:00",  # single range
        "Mo-Fr 08:00-12:00,13:00-20:00",  # multi range
        "Mo-Fr 09:00-21:00; Sa 09:00-13:00; PH off",  # multi rule with public-holiday clause
    ],
)
def test_assert_osm_parses_accepts_valid_plain_osm(osm: str) -> None:
    """Well-formed plain-OSM strings must pass the build gate without raising."""
    assert_osm_parses(osm)


@pytest.mark.parametrize("osm", ["definitely not hours!!!", "Xy 99:99", ""])
def test_assert_osm_parses_rejects_malformed(osm: str) -> None:
    """Malformed strings must raise `ValueError` so the compile step fails the build."""
    with pytest.raises(ValueError):
        assert_osm_parses(osm)


def test_assert_osm_parses_error_names_the_entry() -> None:
    """The build-failure message must name the offending entry to be actionable."""
    with pytest.raises(ValueError, match="5603"):
        assert_osm_parses("definitely not hours!!!", entry_id="5603")
