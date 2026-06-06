import logging

import pydantic
import requests

IRIS_API_URL = "https://iris.asta.tum.de/api/"


class IrisRoom(pydantic.BaseModel):
    """
    A single room from the AStA Iris learning-room API (`GET https://iris.asta.tum.de/api/`).

    Iris is a third party whose response shape we don't control, so unknown fields are
    ignored rather than rejected (e.g. the `percent`/`color` WAAS fields are not modelled here).
    """

    model_config = pydantic.ConfigDict(extra="ignore", str_strip_whitespace=True)

    # The `<arch_name>@<building_id>` form, joined against NavigaTUM aliases.
    raum_nr_architekt: str
    # The NavigaTUM building id (verified 1:1), used as a cross-check on the alias join.
    gebaeude_code: str


def fetch_iris_rooms() -> list[IrisRoom] | None:
    """
    Fetch the Iris learning-room roster once per build.

    Returns the parsed rooms, or `None` if Iris is unreachable or returns an unexpected shape.
    A transient AStA outage must never break the build, so any failure is logged and swallowed;
    the caller falls back to the previously-known coverage set.
    """
    try:
        response = requests.get(IRIS_API_URL, timeout=30)
        response.raise_for_status()
        raeume = response.json()["raeume"]
        return [IrisRoom.model_validate(raum) for raum in raeume]
    except (requests.RequestException, ValueError, KeyError, pydantic.ValidationError) as error:
        logging.warning("Could not fetch Iris learning-room coverage from %s: %s", IRIS_API_URL, error)
        return None
