import json
import sqlite3
from typing import Any, Union


def add_to_database(new_data):
    """add data consisting of (key, de, en) to the sqlite database"""
    con: sqlite3.Connection = sqlite3.connect("data/api_data.db")
    con.execute(
        """
    CREATE TABLE IF NOT EXISTS api_data (
        key     VARCHAR(30) UNIQUE PRIMARY KEY NOT NULL,
        de      BLOB NOT NULL,
        en      BLOB NOT NULL
    );""",
    )
    # we are using this file in docker, so we don't want to use an acid compliant database ;)
    con.execute("""PRAGMA journal_mode = OFF;""")
    con.execute("""PRAGMA synchronous = OFF;""")

    with con:
        con.executemany("INSERT INTO api_data(key, de, en) VALUES (?,?,?)", new_data)


def localise(value: Union[str, list[Any], dict[str, Any]], language: str) -> Any:
    """Recursively localise a dictionary"""
    if isinstance(value, (bool, float, int, str)) or value is None:
        return value
    if isinstance(value, list):
        return [localise(v, language) for v in value]
    if isinstance(value, dict):
        # We consider each dict that has only the keys "de" and/or "en" as translated string
        if set(value.keys()) | {"de", "en"} == {"de", "en"}:
            # Since we only localise strings, the default to the empty string is safe
            return value.get(language, "")

        return {k: localise(v, language) for k, v in value.items()}
    raise ValueError(f"Unhandled type {type(value)}")


def get_localised_data() -> list[tuple[str, str, str]]:
    """get all data from the json dump and convert it to a list of tuples"""
    with open("data/api_data.json", encoding="utf-8") as file:
        data = json.load(file)
    split_data = [(key, localise(value, "de"), localise(value, "en")) for key, value in data.items()]
    return [(key, json.dumps(de), json.dumps(en)) for key, de, en in split_data]


if __name__ == "__main__":
    localised_data = get_localised_data()
    add_to_database(localised_data)
    print("Initialized KV store")
