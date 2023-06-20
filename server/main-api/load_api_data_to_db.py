import dataclasses
import json
import sqlite3
from typing import Any, TypeAlias, Union


def save_entries_to_database(de_data, en_data):
    """add data consisting of 2x(key, data_json, data) to the sqlite database"""
    con: sqlite3.Connection = sqlite3.connect("data/api_data.db")
    for lang in ["de", "en"]:
        con.execute(f"DROP TABLE IF EXISTS {lang}")
        con.execute(
            f"""
            CREATE TABLE {lang} (
                key                 TEXT UNIQUE PRIMARY KEY NOT NULL,
                name                TEXT NOT NULL,
                tumonline_room_nr   INTEGER NULLABLE, -- used for calendars
                type                TEXT NOT NULL,
                type_common_name    TEXT NOT NULL,
                lat                 FLOAT NOT NULL,
                lon                 FLOAT NOT NULL,
                data                TEXT NOT NULL
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
            data["props"].get("tumonline_room_nr", None),
            data["type"],
            data["type_common_name"],
            data.get("coords", {}).get("lat", 48.14903),
            data.get("coords", {}).get("lon", 11.56735),
        )

    de_data = [map_data(key, data_json, data) for (key, data_json, data) in de_data]
    en_data = [map_data(key, data_json, data) for (key, data_json, data) in en_data]

    with con:
        con.executemany(
            "INSERT INTO de(key,data,name,tumonline_room_nr,type,type_common_name,lat,lon) "
            "VALUES (?,?,?,?,?,?,?,?)",
            de_data,
        )
        con.executemany(
            "INSERT INTO en(key,data,name,tumonline_room_nr,type,type_common_name,lat,lon) "
            "VALUES (?,?,?,?,?,?,?,?)",
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


@dataclasses.dataclass
class Alias:
    alias: str
    key: str
    type:str

    def __hash__(self):
        return hash((self.alias,self.key))

def extract_aliases()-> set[Alias]:
    """Extracts all aliases from the api_data.json file and returns them as a dict"""
    with open("data/api_data.json", encoding="utf-8") as file:
        data = json.load(file)
    aliases=set()
    for key,value in data.items():
        if arch_name:=value["arch_name"]:
            aliases.add(Alias(arch_name,key,value["type"]))
    return aliases

def save_aliases_to_database(aliase:set[Alias]):
    con: sqlite3.Connection = sqlite3.connect("data/api_data.db")
    con.execute(f"DROP TABLE IF EXISTS aliases")
    con.execute(
        """
        CREATE TABLE aliases (
            id      INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            alias   TEXT NOT NULL,
            key     TEXT NOT NULL,
            type    TEXT NOT NULL
        );""",
    )
    with con:
        con.executemany(
            "INSERT INTO aliases(alias,key,type)"
            "VALUES (?,?,?)",
            [(item.alias,item.key,item.type) for item in aliase],
        )

if __name__ == "__main__":
    de, en = get_localised_data()
    save_entries_to_database(de, en)
    print("Initialized KV store")
    extracted_aliases=extract_aliases()
    save_aliases_to_database(extracted_aliases)
    print("Initialized alias store")

