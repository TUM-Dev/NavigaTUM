import json
import sqlite3
from typing import Any, Union


def add_to_database(de_data, en_data):
    """add data consisting of 2x(key, data_json, data) to the sqlite database"""
    con: sqlite3.Connection = sqlite3.connect("data/api_data.db")
    con.execute(
        """
    CREATE TABLE IF NOT EXISTS de (
        key                 VARCHAR(30) UNIQUE PRIMARY KEY NOT NULL,
        name                VARCHAR(30),
        type                VARCHAR(30),
        type_common_name    VARCHAR(30),
        lat                 FLOAT,
        lon                 FLOAT,
        data                BLOB NOT NULL
    );""",
    )
    con.execute(
        """
    CREATE TABLE IF NOT EXISTS en (
        key                 VARCHAR(30) UNIQUE PRIMARY KEY NOT NULL,
        name                VARCHAR(30),
        type                VARCHAR(30),
        type_common_name    VARCHAR(30),
        lat                 FLOAT,
        lon                 FLOAT,
        data                BLOB NOT NULL
    );""",
    )
    # we are using this file in docker, so we don't want to use an acid compliant database ;)
    con.execute("""PRAGMA journal_mode = OFF;""")
    con.execute("""PRAGMA synchronous = OFF;""")

    def map_data(key, data_json, data):
        return (
            key,
            data_json,
            data["name"],
            data["type"],
            data["type_common_name"],
            data.get("coords", {}).get("lat", 48.14903),
            data.get("coords", {}).get("lon", 11.56735),
        )

    de_data = [map_data(key, data_json, data) for (key, data_json, data) in de_data]
    en_data = [map_data(key, data_json, data) for (key, data_json, data) in en_data]

    with con:
        con.executemany("INSERT INTO de(key, data, name,type,type_common_name,lat,lon) VALUES (?,?,?,?,?,?,?)", de_data)
        con.executemany("INSERT INTO en(key, data, name,type,type_common_name,lat,lon) VALUES (?,?,?,?,?,?,?)", en_data)


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


translated_list = list[tuple[str, str, Any]]


def get_localised_data() -> tuple[translated_list, translated_list]:
    """get all data from the json dump and convert it to a list of tuples"""
    with open("data/api_data.json", encoding="utf-8") as file:
        data = json.load(file)
    split_data: list[tuple[str, Any, Any]] = [
        (key, localise(value, "de"), localise(value, "en")) for key, value in data.items()
    ]

    de_data = []
    en_data = []
    for key, de_dict, en_dict in split_data:
        de_data.append((key, json.dumps(de_dict), de_dict))
        en_data.append((key, json.dumps(en_dict), en_dict))
    return de_data, en_data


if __name__ == "__main__":
    de, en = get_localised_data()
    add_to_database(de, en)
    print("Initialized KV store")
