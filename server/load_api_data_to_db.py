import json
import sqlite3
from typing import Any, TypeAlias, Union


def add_to_database(de_data, en_data):
    """add data consisting of 2x(key, data_json, data) to the sqlite database"""
    con: sqlite3.Connection = sqlite3.connect("data/api_data.db")
    for lang in ["de", "en"]:
        con.execute(f"DROP TABLE IF EXISTS {lang}")
        con.execute(
            f"""
            CREATE TABLE {lang} (
                key                 VARCHAR(30) UNIQUE PRIMARY KEY NOT NULL,
                name                VARCHAR(30) NOT NULL,
                tumonline_room_nr   INTEGER NULLABLE, -- used for calendars
                arch_name           VARCHAR(30), -- NOT Unique, but only (temporarily) used for the old roomfinder.
                type                VARCHAR(30) NOT NULL,
                type_common_name    VARCHAR(30) NOT NULL,
                lat                 FLOAT NOT NULL,
                lon                 FLOAT NOT NULL,
                data                TEXT NOT NULL
            );""",
        )
    for tbl in ["calendar", "calendar_scrape"]:
        con.execute(f"DROP TABLE IF EXISTS {tbl}")
        con.execute(
            f"""
        CREATE TABLE {tbl} (
            key                     VARCHAR(30) NOT NULL,
            dtstart                 DATETIME NOT NULL,
            dtend                   DATETIME NOT NULL,
            dtstamp                 DATETIME NOT NULL,
            event_id                INTEGER NOT NULL,
            event_title             TEXT NOT NULL,
            single_event_id         INTEGER UNIQUE PRIMARY KEY NOT NULL,
            single_event_type_id    TEXT NOT NULL,
            single_event_type_name  TEXT NOT NULL,
            event_type_id           TEXT NOT NULL,
            event_type_name         TEXT NULLABLE,
            course_type_name        TEXT NULLABLE,
            course_type             TEXT NULLABLE,
            course_code             TEXT NULLABLE,
            course_semester_hours   INTEGER NULLABLE,
            group_id                TEXT NULLABLE,
            xgroup                  TEXT NULLABLE,
            status_id               TEXT NOT NULL,
            status                  TEXT NOT NULL,
            comment                 TEXT NOT NULL
        );""",
        )
    # purposely, this index is only on this table and not on calendar_scrape
    con.execute("CREATE INDEX IF NOT EXISTS calendar_lut ON calendar(key, dtstart, dtend)")
    # we are using this file in docker, so we don't want to use an acid compliant database ;)
    con.execute("""PRAGMA journal_mode = OFF;""")
    con.execute("""PRAGMA synchronous = OFF;""")

    def map_data(key, data_json, data):
        return (
            key,
            data_json,
            data["name"],
            data["props"].get("tumonline_room_nr", None),
            data["arch_name"],
            data["type"],
            data["type_common_name"],
            data.get("coords", {}).get("lat", 48.14903),
            data.get("coords", {}).get("lon", 11.56735),
        )

    de_data = [map_data(key, data_json, data) for (key, data_json, data) in de_data]
    en_data = [map_data(key, data_json, data) for (key, data_json, data) in en_data]

    with con:
        con.executemany(
            "INSERT INTO de(key,data,name,tumonline_room_nr,arch_name,type,type_common_name,lat,lon) "
            "VALUES (?,?,?,?,?,?,?,?,?)",
            de_data,
        )
        con.executemany(
            "INSERT INTO en(key,data,name,tumonline_room_nr,arch_name,type,type_common_name,lat,lon) "
            "VALUES (?,?,?,?,?,?,?,?,?)",
            en_data,
        )


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


TranslatedList: TypeAlias = list[tuple[str, str, Any]]


def get_localised_data() -> tuple[TranslatedList, TranslatedList]:
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
