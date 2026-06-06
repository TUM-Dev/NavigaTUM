import polars as pl

from external.models.common import RESULTS_PATH, PydanticConfiguration


class IrisRoom(PydanticConfiguration):
    """
    A learning room from the AStA Iris roster (`GET https://iris.asta.tum.de/api/`).

    Only the two fields the build-time coverage join needs are stored; the volatile `status`
    and WAAS `percent`/`color` fields are runtime concerns handled elsewhere.
    """

    # The `<arch_name>@<building_id>` form, joined against NavigaTUM aliases.
    raum_nr_architekt: str
    # The NavigaTUM building id (verified 1:1), used as a cross-check on the alias join.
    gebaeude_code: str

    @classmethod
    def load_all(cls) -> list["IrisRoom"]:
        """Load every room from the stored roster."""
        # Building ids like "0201" must stay strings so leading zeros survive the round-trip.
        df = pl.read_csv(
            RESULTS_PATH / "iris.csv",
            schema_overrides={"raum_nr_architekt": pl.String, "gebaeude_code": pl.String},
        )
        return [cls(**row) for row in df.iter_rows(named=True)]
