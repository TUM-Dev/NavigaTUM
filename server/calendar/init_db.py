import sqlite3
from pathlib import Path

DATA_DIR = Path(__file__).parent / "data"
DATA_DIR.mkdir(exist_ok=True)


def init_db():
    """
    Initialize the database with the tables and indices we need.
    """
    con: sqlite3.Connection = sqlite3.connect(DATA_DIR / "calendar_data.db")
    con.execute("DROP TABLE IF EXISTS calendar")
    con.execute(
        """
    CREATE TABLE calendar (
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
        comment                 TEXT NOT NULL,
        last_scrape             DATETIME NOT NULL
    );""",
    )
    con.execute("CREATE INDEX IF NOT EXISTS calendar_lut ON calendar(key, dtstart, dtend)")


if __name__ == "__main__":
    init_db()
    print("Initialized Calendar store")
