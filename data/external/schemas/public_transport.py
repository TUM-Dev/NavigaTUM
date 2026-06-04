import dataframely as dy


class StationsSchema(dy.Schema):
    """
    Schema for the public-transport station catalogue (`public_transport.parquet`).

    Source: DELFI/MVV GTFS export (zHV-Gesamt). Column order matches the parquet
    written by `external.scrapers.public_transport.scrape_stations`.
    """

    dhid = dy.String(nullable=False)
    parent = dy.String(nullable=True)
    name = dy.String(nullable=False)
    lat = dy.Float32(nullable=False)
    lon = dy.Float32(nullable=False)
    is_sev = dy.Bool(nullable=False)
