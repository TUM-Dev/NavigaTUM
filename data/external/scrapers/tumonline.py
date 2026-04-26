import logging
import os
import typing

import backoff
import polars as pl
import requests
from oauthlib.oauth2 import BackendApplicationClient
from requests_oauthlib import OAuth2Session

from external.schemas.tumonline import BuildingsSchema, OrgsSchema, UsagesSchema
from external.scraping_utils import CACHE_PATH
from utils import setup_logging

TUMONLINE_URL = "https://campus.tum.de/tumonline"
CONNECTUM_URL = f"{TUMONLINE_URL}/co/connectum"


def _generate_oauth_headers() -> dict[str, str]:
    """
    Generate the OAuth token and packs it into a header form for easier consumption.

    This is safe, because the token is valid for 300s and no downloading will take more than this amount.
    """
    token_url = f"{TUMONLINE_URL}/co/public/sec/auth/realms/CAMPUSonline_SP/protocol/openid-connect/token"
    oauth_client_id = os.environ.get("CONNECTUM_OAUTH_CLIENT_ID")
    assert oauth_client_id is not None, "requests to connectum need CONNECTUM_OAUTH_CLIENT_ID specified"
    oauth_client_secret = os.environ.get("CONNECTUM_OAUTH_CLIENT_SECRET")
    assert oauth_client_secret is not None, "requests to connectum need CONNECTUM_OAUTH_CLIENT_SECRET specified"

    oauth_client = BackendApplicationClient(client_id=oauth_client_id)
    oauth_session = OAuth2Session(client=oauth_client)
    token = oauth_session.fetch_token(
        token_url=token_url,
        client_id=oauth_client_id.strip(),
        client_secret=oauth_client_secret.strip(),
    )
    assert token is not None

    return {"Authorization": f"Bearer {token['access_token']}"}


OAUTH_HEADERS = _generate_oauth_headers()


@backoff.on_exception(backoff.expo, requests.exceptions.RequestException)
def scrape_buildings() -> None:
    """Retrieve the buildings as in TUMonline"""
    logging.info("Downloading the buildings of tumonline")

    payload = requests.get(f"{CONNECTUM_URL}/api/rooms/buildings", headers=OAUTH_HEADERS, timeout=30).json()
    rows = [
        {
            "building_key": f"{b['building_id']:04d}",
            "address_place": b["address_place"],
            "address_street": b["address_street"],
            "address_zip_code": b["address_zip_code"],
            "area_id": b["area_id"],
            "name": b["name"],
            "tumonline_id": b["nr"],
            "filter_id": b.get("filter_id"),
        }
        for b in payload
    ]
    df = pl.DataFrame(rows, schema=BuildingsSchema.to_polars_schema()).sort("building_key")
    df.write_csv(CACHE_PATH / "buildings_tumonline.csv")


@backoff.on_exception(backoff.expo, requests.exceptions.RequestException)
def scrape_rooms() -> None:
    """Retrieve the rooms as in TUMonline"""
    logging.info("Downloading the rooms of tumonline")

    def _sanitise_room_value(val: dict[str, typing.Any]) -> dict[str, typing.Any]:
        val["tumonline_id"] = val.pop("nr")  # tumonline id for this room, not really relevant in our context
        val.pop("room_code")
        val["address"] = {
            "place": val.pop("address_place"),
            "street": val.pop("address_street"),
            "zip_code": val.pop("address_zip_code"),
        }
        if "alt_name" in val:
            val["alt_name"] = _clean_spaces(val["alt_name"]).replace(" ( ", " (")
        val["floor_level"] = val.pop("address_floor")
        val["seats"] = {
            "sitting": val.pop("seats", None),
            "wheelchair": val.pop("wheelchair_seats", None),
            "standing": val.pop("standing_seats", None),
        }
        return val

    rooms = requests.get(f"{CONNECTUM_URL}/api/rooms", headers=OAUTH_HEADERS, timeout=30).json()
    rooms = {r["room_code"]: _sanitise_room_value(r) for r in rooms}

    # Convert to CSV format
    rows = []
    for room_key, room_data in rooms.items():
        address = room_data.get("address", {})
        seats = room_data.get("seats", {})

        row = {
            "room_key": str(room_key),
            "address_place": str(address.get("place", "")),
            "address_street": str(address.get("street", "")),
            "address_zip_code": int(address.get("zip_code", 0)),
            "seats_sitting": seats.get("sitting") if seats.get("sitting") is not None else None,
            "seats_wheelchair": seats.get("wheelchair") if seats.get("wheelchair") is not None else None,
            "seats_standing": seats.get("standing") if seats.get("standing") is not None else None,
            "floor_type": str(room_data.get("floor_type", "")),
            "floor_level": str(room_data.get("floor_level", "")),
            "tumonline_id": int(room_data.get("tumonline_id", 0)),
            "area_id": int(room_data.get("area_id", 0)),
            "building_id": int(room_data.get("building_id", 0)),
            "main_operator_id": int(room_data.get("main_operator_id", 0)),
            "usage_id": int(room_data.get("usage_id", 0)),
            "alt_name": str(room_data.get("alt_name", "")) if room_data.get("alt_name") else None,
            "arch_name": str(room_data.get("arch_name", "")) if room_data.get("arch_name") else None,
            "calendar_resource_nr": room_data.get("calendar_resource_nr")
            if room_data.get("calendar_resource_nr") is not None
            else None,
            "patched": bool(room_data.get("patched", False)),
        }
        rows.append(row)

    df = pl.DataFrame(rows, infer_schema_length=None)
    # Sort by room_key for consistency
    df = df.sort("room_key")
    df.write_csv(CACHE_PATH / "rooms_tumonline.csv")


def _clean_spaces(_string: str) -> str:
    """Remove leading and trailing spaces as well as duplicate spaces in-between"""
    return " ".join(_string.split())


@backoff.on_exception(backoff.expo, requests.exceptions.RequestException)
def scrape_usages() -> None:
    """Retrieve all usage types available in TUMonline."""
    logging.info("Downloading the usage types of tumonline")

    payload = requests.get(f"{CONNECTUM_URL}/api/rooms/usages", headers=OAUTH_HEADERS, timeout=30).json()

    rows = [{"usage_id": u["id"], **{c: u[c] for c in UsagesSchema.column_names() if c != "usage_id"}} for u in payload]
    df = pl.DataFrame(rows, schema=UsagesSchema.to_polars_schema()).sort("usage_id")
    df.write_csv(CACHE_PATH / "usages_tumonline.csv")


@backoff.on_exception(backoff.expo, requests.exceptions.RequestException)
def scrape_orgs(lang: typing.Literal["de", "en"]) -> None:
    """
    Retrieve all organisations in TUMonline, that may operate rooms.

    :params lang: 'en' or 'de'
    """
    logging.info("Scraping the orgs of tumonline")

    # There is also this URL, which is used to retrieve orgs that have courses,
    # but this is not merged in at the moment:
    # https://campus.tum.de/tumonline/ee/rest/brm.orm.search/organisations/chooser?$language=de&view=S_COURSE_LVEAB_ORG
    url = f"{TUMONLINE_URL}/ee/rest/brm.orm.search/organisations?q=*&$language={lang}"

    req = requests.get(url, headers={"Accept": "application/json"}, timeout=30)
    assert req.status_code == 200, f"Failed to download organisations.\n{req=}\n{req.text=}"

    rows = [
        {
            "org_id": dto["id"],
            "code": dto["designation"],
            "name": dto["name"],
            "path": dto["orgPath"],
        }
        for dto in (resource["content"]["organisationSearchDto"] for resource in req.json()["resource"])
        if dto.get("designation")
    ]
    df = pl.DataFrame(rows, schema=OrgsSchema.to_polars_schema()).sort("org_id")
    df.write_csv(CACHE_PATH / f"orgs-{lang}_tumonline.csv")


if __name__ == "__main__":
    setup_logging(level=logging.INFO)
    scrape_buildings()
    scrape_rooms()
    scrape_usages()
    scrape_orgs(lang="de")
    scrape_orgs(lang="en")
