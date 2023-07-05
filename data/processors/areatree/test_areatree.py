import logging
import tempfile
import unittest
from pathlib import Path

from processors import areatree
from processors.areatree.process import (
    _areatree_lines,
    _extract_building_prefix,
    _extract_id_and_type,
    _extract_names,
    _split_line,
)


class AreatreeExtractNames(unittest.TestCase):
    def test_extract_names_with_short_name(self):
        names = ["Mathematics Informatics", "mi"]
        expected_output = {"name": "Mathematics Informatics", "short_name": "mi"}
        self.assertEqual(_extract_names(names), expected_output)

    def test_extract_names_without_short_name(self):
        names = ["Mathematics Informatics"]
        expected_output = {"name": "Mathematics Informatics"}
        self.assertEqual(_extract_names(names), expected_output)

    def test_extract_names_with_long_short_name(self):
        names = ["Mechanical Engineering", "ThisIsAVeryLongNameForAShortName"]
        expected_output = {"name": "Mechanical Engineering", "short_name": "ThisIsAVeryLongNameForAShortName"}
        with self.assertLogs(level=logging.WARNING) as cm:
            self.assertEqual(_extract_names(names), expected_output)
            self.assertIn("'ThisIsAVeryLongNameForAShortName' is very long for a short name (>20 chars)", cm.output[0])

    def test_extract_names_with_extra_names(self):
        names = ["Name1", "Name2", "Name3"]
        with self.assertRaises(RuntimeError):
            _extract_names(names)
        with self.assertRaises(IndexError):
            _extract_names([])


class AreatreeExtractBuildingPrefix(unittest.TestCase):
    def test_dash_separator(self):
        expected_result = {"b_prefix": "b1-b2-b3"}
        self.assertEqual(_extract_building_prefix("b1-b2-b3"), expected_result)

    def test_areatree_uncertain(self):
        expected_result = {"data_quality": {"areatree_uncertain": True}, "b_prefix": "b1-b2"}
        self.assertEqual(_extract_building_prefix("-b1-b2"), expected_result)

    def test_comma_separator(self):
        expected_result = {"b_prefix": ["b1", "b2", "b3"]}
        self.assertEqual(_extract_building_prefix("b1,b2,b3"), expected_result)

    def test_empty(self):
        self.assertEqual(_extract_building_prefix(""), {})

    def test_building_ids_without_separator(self):
        expected_result = {"b_prefix": "b1"}
        self.assertEqual(_extract_building_prefix("b1"), expected_result)


class AreatreeExtractIdAndType(unittest.TestCase):
    def test_specified_type(self):
        expected = {"id": "abc", "type": "building"}
        self.assertEqual(_extract_id_and_type("abc[building]", None), expected)
        self.assertEqual(_extract_id_and_type("abc[building]", "cdf"), expected)

    def test_comma(self):
        expected = {"id": "123", "visible_id": "visible_id", "type": "area"}
        self.assertEqual(_extract_id_and_type("123,visible_id", None), expected)
        self.assertEqual(_extract_id_and_type("123,visible_id", "cdf"), expected)

    def test_single_id(self):
        expected = {"id": "xyz", "type": "building"}
        self.assertEqual(_extract_id_and_type("xyz", "xyz"), expected)

    def test_id_not_inferable(self):
        with self.assertRaises(RuntimeError):
            _extract_id_and_type("", ["b_prefix1", "b_prefix2"])
        with self.assertRaises(RuntimeError):
            _extract_id_and_type("123,visible_id,extra_id", ["b_prefix1", "b_prefix2"])
        with self.assertRaises(RuntimeError):
            _extract_id_and_type("123,visible_id,extra_id", None)


class AreatreeLinesTestCase(unittest.TestCase):
    def test_empty_file(self):
        with tempfile.NamedTemporaryFile() as file:
            areatree.process.AREATREE_FILE = Path(file.name)
            self.assertEqual(list(areatree.process._areatree_lines()), [])

    def test_comment_lines(self):
        with tempfile.NamedTemporaryFile(mode="w+") as file:
            areatree.process.AREATREE_FILE = Path(file.name)
            file.write("line1\n")
            file.write("\n")  # Empty line
            file.write("# Comment line\n")
            file.write("line2\n")
            file.flush()
            self.assertEqual(list(areatree.process._areatree_lines()), ["line1", "line2"])

    def test_inline_comments(self):
        with tempfile.NamedTemporaryFile(mode="w+") as file:
            areatree.process.AREATREE_FILE = Path(file.name)
            file.write("line1#comment1\n")
            file.write("line2#comment2 # comment 3\n")
            file.flush()
            self.assertEqual(list(areatree.process._areatree_lines()), ["line1", "line2"])

    def test_file_preserves_indentation(self):
        with tempfile.NamedTemporaryFile(mode="w+") as file:
            areatree.process.AREATREE_FILE = Path(file.name)
            file.write("  line1  \n")
            file.write(" line2\n")
            file.write("line3")
            file.flush()
            self.assertEqual(list(areatree.process._areatree_lines()), ["  line1", " line2", "line3"])


class SplitLineTestCase(unittest.TestCase):
    def test_valid_line(self):
        self.assertEqual(_split_line("1:Building A:123,456"), ("1", "Building A", "123,456"))

    def test_invalid_line_missing_parts(self):
        with self.assertRaises(RuntimeError):
            _split_line("1:Building A")

    def test_invalid_line_extra_parts(self):
        with self.assertRaises(RuntimeError):
            _split_line("1:Building A:123,456:extra_part")


if __name__ == "__main__":
    unittest.main()
