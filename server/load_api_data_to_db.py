import json
import sqlite3

con: sqlite3.Connection = sqlite3.connect("data/api_data.db")
con.execute(
    """
CREATE TABLE IF NOT EXISTS api_data (
    key     VARCHAR(30) UNIQUE PRIMARY KEY NOT NULL,
    value   BLOB NOT NULL
);""",
)
# we are using this file in docker, so we don't want to use an acid compliant database ;)
con.execute("""PRAGMA journal_mode = OFF;""")
con.execute("""PRAGMA synchronous = OFF;""")

with open("data/api_data.json") as f:
    data = json.load(f)
    new_data: list[tuple[str, str]] = [(key, json.dumps(value)) for key, value in data.items()]

with con:
    con.executemany("INSERT INTO api_data(key, value) VALUES (?,?)", new_data)
print("Initialized KV store")
