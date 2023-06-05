import difflib
import itertools
from pathlib import Path

import yaml

HOME = Path(__file__).parent


def recursively_simplify(prefix: list[str], item_dict: dict) -> list[str]:
    """
    Recursively simplifies a dictionary to a list of strings.
    Each string is a path to a leaf in the dictionary.
    @param prefix: the path to the current item
    @param item_dict: the current item
    """
    results: list[str] = []
    for key, value in item_dict.items():
        if isinstance(value, dict):
            results += recursively_simplify(prefix + [key], value)
        else:
            results.append(".".join(prefix + [key]))
    return sorted(results)


def parse_files() -> list[tuple[str, list[str]]]:
    """
    Parses all yaml files in the current directory and returns them as a list of tuples.
    Each tuple contains the filename and the sorted list of keys in the file.
    """
    files = []
    for file_path in HOME.glob("*.yaml"):
        with open(file_path, encoding="utf-8") as file:
            item_dict = yaml.unsafe_load(file)
            files.append((file_path.stem, recursively_simplify([], item_dict)))
        # making sure that all files are written in the same style (sorted, avoiding special characters)
        with open(file_path, "w", encoding="utf-8") as file:
            yaml.dump(item_dict, file, sort_keys=True, allow_unicode=True, width=1000)
    return files


def diff_files(files: list[tuple[str, list[str]]]):
    """
    Compares all files in the list via difflib and prints the differences.
    Raises an error if any two files have different content
    """
    for (filename1, content1), (filename2, content2) in itertools.combinations(files, 2):
        if content1 != content2:
            print(f"{filename1} and {filename2} have different content:")
            for line in difflib.context_diff(content1, content2, fromfile=filename1, tofile=filename2):
                print(line.strip())
    if not all(c1 == c2 for _, c1 in files for _, c2 in files):
        raise ValueError("Files have different content")


if __name__ == "__main__":
    diff_files(parse_files())
