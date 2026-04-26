import logging
import tempfile
from pathlib import Path
from typing import Any

import pytest
from external.loaders.tumonline import load_buildings
from processors import areatree
from processors.areatree.process import (
    _areatree_lines,
    _extract_building_prefix,
    _extract_id_and_type,
    _extract_names,
    _split_line,
)

LOGGER = logging.getLogger(__name__)


def test_extract_names_with_short_name() -> None:
    """If there is a short name, it is returned as well"""
    names = ["Mathematics Informatics", "mi"]
    expected_output = {"name": "Mathematics Informatics", "short_name": "mi"}
    assert _extract_names(names) == expected_output


def test_extract_names_without_short_name() -> None:
    """If there is no short name, only the name is returned"""
    names = ["Mathematics Informatics"]
    expected_output = {"name": "Mathematics Informatics"}
    assert _extract_names(names) == expected_output


def test_extract_names_with_long_short_name(caplog: Any) -> None:
    """If the short name is longer than 20 chars, a warning is raised"""
    names = ["Mechanical Engineering", "ThisIsAVeryLongNameForAShortName"]
    expected_output = {"name": "Mechanical Engineering", "short_name": "ThisIsAVeryLongNameForAShortName"}
    with caplog.at_level(logging.WARNING):
        assert _extract_names(names) == expected_output
    assert "'ThisIsAVeryLongNameForAShortName' is very long for a short name (>20 chars)" in caplog.text


def test_extract_names_with_extra_names() -> None:
    """If there are more than two names, an error is raised"""
    names = ["Name1", "Name2", "Name3"]
    with pytest.raises(RuntimeError):
        _extract_names(names)
    with pytest.raises(IndexError):
        _extract_names([])


def test_dash_separator() -> None:
    """If the building id is separated by a dash, it is returned as a string"""
    expected_result = {"b_prefix": "b1-b2-b3"}
    assert _extract_building_prefix("b1-b2-b3") == expected_result


def test_areatree_uncertain() -> None:
    """If the building id starts with a dash, it is marked as uncertain"""
    expected_result = {"data_quality": {"areatree_uncertain": True}, "b_prefix": "b1-b2"}
    assert _extract_building_prefix("-b1-b2") == expected_result


def test_comma_separator() -> None:
    """If the building id is separated by a comma, it is split into a list"""
    expected_result = {"b_prefix": ["b1", "b2", "b3"]}
    assert _extract_building_prefix("b1,b2,b3") == expected_result


def test_empty() -> None:
    """If the building id is empty, an empty dict is returned"""
    assert not _extract_building_prefix("")


def test_building_ids_without_separator() -> None:
    """If the building id is not separated by a dash or comma, it is returned as is"""
    assert _extract_building_prefix("b1") == {"b_prefix": "b1"}


def test_specified_type() -> None:
    """If the type is specified, it is returned"""
    expected = {"id": "abc", "type": "building"}
    assert _extract_id_and_type("abc[building]", None) == expected
    assert _extract_id_and_type("abc[building]", "cdf") == expected


def test_comma_specified_type() -> None:
    """If the building id is separated by a comma, it is split into a list"""
    expected = {"id": "abc", "type": "building", "visible_id": "bcd"}
    with pytest.raises(RuntimeError):
        assert _extract_id_and_type("abc[building],bcd", None)
    assert _extract_id_and_type("abc,bcd[building]", None) == expected
    assert _extract_id_and_type("abc,bcd[building]", "cdf") == expected


def test_comma() -> None:
    """If the id is inferable from the line, it is returned"""
    expected = {"id": "123", "visible_id": "visible_id", "type": "area"}
    assert _extract_id_and_type("123,visible_id", None) == expected
    assert _extract_id_and_type("123,visible_id", "cdf") == expected


def test_single_id() -> None:
    """If the id is inferable from the line, it is returned"""
    expected = {"id": "xyz", "type": "building"}
    assert _extract_id_and_type("xyz", "xyz") == expected


def test_id_not_inferable() -> None:
    """If the id is not inferable from the line, an error is raised"""
    with pytest.raises(RuntimeError):
        _extract_id_and_type("", ["b_prefix1", "b_prefix2"])
    with pytest.raises(RuntimeError):
        _extract_id_and_type("123,visible_id,extra_id", ["b_prefix1", "b_prefix2"])
    with pytest.raises(RuntimeError):
        _extract_id_and_type("123,visible_id,extra_id", None)


@pytest.fixture
def areatree_tempfile() -> Any:
    """Bind AREATREE_FILE to a writable temp file; restore the original on teardown."""
    original = areatree.process.AREATREE_FILE
    with tempfile.NamedTemporaryFile(mode="w+") as file:
        areatree.process.AREATREE_FILE = Path(file.name)
        yield file
    areatree.process.AREATREE_FILE = original


def test_empty_file(areatree_tempfile: Any) -> None:
    """Empty file returns empty list"""
    assert not list(_areatree_lines())


def test_comment_lines(areatree_tempfile: Any) -> None:
    """Comment lines are removed"""
    areatree_tempfile.write("line1\n")
    areatree_tempfile.write("\n")  # Empty line
    areatree_tempfile.write("# Comment line\n")
    areatree_tempfile.write("line2\n")
    areatree_tempfile.flush()
    assert list(_areatree_lines()) == ["line1", "line2"]


def test_inline_comments(areatree_tempfile: Any) -> None:
    """Inline comments are removed"""
    areatree_tempfile.write("line1#comment1\n")
    areatree_tempfile.write("line2#comment2 # comment 3\n")
    areatree_tempfile.flush()
    assert list(_areatree_lines()) == ["line1", "line2"]


def test_file_preserves_indentation(areatree_tempfile: Any) -> None:
    """Indentation is preserved"""
    areatree_tempfile.write("  line1  \n")
    areatree_tempfile.write(" line2\n")
    areatree_tempfile.write("line3")
    areatree_tempfile.flush()
    assert list(_areatree_lines()) == ["  line1", " line2", "line3"]


def test_valid_line() -> None:
    """Valid lines are split correctly"""
    assert _split_line("1:Building A:123,456") == ("1", "Building A", "123,456")


def test_invalid_line_missing_parts() -> None:
    """Missing parts are not allowed"""
    with pytest.raises(RuntimeError):
        _split_line("1:Building A")


def test_invalid_line_extra_parts() -> None:
    """Extra parts are not allowed"""
    with pytest.raises(RuntimeError):
        _split_line("1:Building A:123,456:extra_part")


def test_all_tumonline_buildings_in_areatree() -> None:
    """Every TUMonline building must have a matching entry in the areatree"""
    df = areatree.process.read_areatree()

    known_b_prefixes: set[str] = set()
    for row in df.iter_rows(named=True):
        if row.get("b_prefix") is not None:
            known_b_prefixes.add(row["b_prefix"])
        if row.get("b_prefix_list") is not None:
            known_b_prefixes.update(row["b_prefix_list"])

    missing = sorted(
        (building["building_key"], building["name"])
        for building in load_buildings().iter_rows(named=True)
        if building["building_key"] not in known_b_prefixes
    )

    assert not missing, "TUMonline buildings missing from areatree:\n" + "\n".join(
        f"  {b_id}: {name}" for b_id, name in missing
    )
