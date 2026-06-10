import logging
import os
import typing

import backoff
import polars as pl
import requests
from oauthlib.oauth2 import BackendApplicationClient, OAuth2Error
from requests_oauthlib import OAuth2Session
from utils import setup_logging

from external.schemas.tumonline import BuildingsSchema, OrgsSchema, RoomsSchema, UsagesSchema
from external.scraping_utils import CACHE_PATH

_logger = logging.getLogger(__name__)

TUMONLINE_URL = "https://campus.tum.de/tumonline"
CONNECTUM_URL = f"{TUMONLINE_URL}/co/connectum"


@backoff.on_exception(backoff.expo, (requests.exceptions.RequestException, OAuth2Error), max_tries=8)
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
    _logger.info("Downloading the buildings of tumonline")

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
    _logger.info("Downloading the rooms of tumonline")

    payload = requests.get(f"{CONNECTUM_URL}/api/rooms", headers=OAUTH_HEADERS, timeout=30).json()
    rows = [
        {
            "room_key": r["room_code"],
            "address_place": r["address_place"],
            "address_street": r["address_street"],
            "address_zip_code": r["address_zip_code"],
            "seats_sitting": r.get("seats"),
            "seats_wheelchair": r.get("wheelchair_seats"),
            "seats_standing": r.get("standing_seats"),
            "floor_type": r["floor_type"],
            "floor_level": r["address_floor"],
            "tumonline_id": r["nr"],
            "area_id": r["area_id"],
            "building_id": r["building_id"],
            "main_operator_id": r["main_operator_id"],
            "usage_id": r["usage_id"],
            "alt_name": _clean_spaces(r["alt_name"]).replace(" ( ", " (") if r.get("alt_name") else None,
            "arch_name": r.get("arch_name") or None,
            "calendar_resource_nr": r.get("calendar_resource_nr"),
            "patched": False,
        }
        for r in payload
    ]
    df = pl.DataFrame(rows, schema=RoomsSchema.to_polars_schema()).sort("room_key")
    # TUMonline occasionally returns the same room twice in the payload (same
    # `tumonline_id`) and, more rarely, two genuinely different rooms sharing
    # the same `room_code` (different `tumonline_id`s — see e.g. 0103.Z1.302).
    # `RoomsSchema` marks `room_key` as the primary key and downstream code
    # keys dictionaries by it, so duplicates must be collapsed before we
    # commit the CSV. `keep="last"` is opinionated — it loses one of two
    # conflicting rooms — but it's deterministic and matches the existing
    # main most of the time. Conflicts are logged so they're visible.
    dup_keys = (
        df.group_by("room_key")
        .agg(pl.col("tumonline_id").unique().alias("ids"))
        .filter(pl.col("ids").list.len() > 1)
    )
    for row in dup_keys.iter_rows(named=True):
        _logger.warning(
            "TUMonline returned conflicting rooms for room_code %r "
            "(tumonline_ids %s); keeping last occurrence.",
            row["room_key"],
            sorted(row["ids"]),
        )
    df = df.unique(subset=["room_key"], keep="last", maintain_order=True).sort("room_key")
    df.write_csv(CACHE_PATH / "rooms_tumonline.csv")


def _clean_spaces(_string: str) -> str:
    """Remove leading and trailing spaces as well as duplicate spaces in-between"""
    return " ".join(_string.split())


@backoff.on_exception(backoff.expo, requests.exceptions.RequestException)
def scrape_usages() -> None:
    """Retrieve all usage types available in TUMonline."""
    _logger.info("Downloading the usage types of tumonline")

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
    _logger.info("Scraping the orgs of tumonline")

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
