import polars as pl

from external.models.common import PydanticConfiguration, RESULTS_PATH


class Station(PydanticConfiguration):
    dhid: str
    parent: str | None
    name: str
    lat: float
    lon: float
    is_sev: bool

    @classmethod
    def load_all(cls) -> list["Station"]:
        """Load all public_transport stations from parquet format"""
        target = RESULTS_PATH / "public_transport.parquet"

        # Load the parquet file
        df = pl.read_parquet(target)

        # Convert to Station objects
        stations = []
        for row in df.iter_rows(named=True):
            station = cls(
                dhid=row["dhid"],
                parent=row["parent"],
                name=row["name"],
                lat=float(row["lat"]),
                lon=float(row["lon"]),
                is_sev=row["is_sev"],
            )
            stations.append(station)

        return stations
