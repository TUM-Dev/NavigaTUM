import json
import logging
import xml.etree.ElementTree as ET  # nosec: used for writing files, defusedxml only supports parse()
from datetime import datetime, timezone
from pathlib import Path
from typing import Literal, TypedDict

import backoff
import requests
from defusedxml import ElementTree as defusedET

OLD_DATA_URL = "https://nav.tum.de/cdn/api_data.json"


class SitemapEntry(TypedDict):
    url: str
    lastmod: datetime
    priority: float


class Sitemaps(TypedDict):
    room: list[SitemapEntry]
    other: list[SitemapEntry]


class SimplifiedSitemaps(TypedDict):
    room: dict[str, datetime]
    other: dict[str, datetime]


OUTPUT_DIR_PATH = Path(__file__).parent.parent / "output"


def generate_sitemap() -> None:
    """Generate a sitemap that diffs changes since to the currently online data"""
    # Load exported data. This function is intentionally not using the data object
    # directly, but re-parsing the output file instead, because the export not
    # export all fields. This way we're also guaranteed to have the same types
    # (and not e.g. numpy floats).
    with (OUTPUT_DIR_PATH / "api_data.json").open(encoding="utf-8") as file:
        new_data: list = json.load(file)

    # Look whether there are currently online sitemaps for the provided
    # sitemaps name. In case there aren't, we assume this sitemap is new,
    # and all entries will be marked as changed
    old_sitemaps = _download_online_sitemaps()
    try:
        old_data = _download_old_data()
    except requests.exceptions.RequestException as error:
        logging.warning(f"Could not download online data because of {error}. Assuming all entries are new.")
        old_data = []

    sitemaps: Sitemaps = _extract_sitemap_data(new_data, old_data, old_sitemaps)

    for name, sitemap in sitemaps.items():
        _write_sitemap_xml(OUTPUT_DIR_PATH / f"sitemap-data-{name}.xml", sitemap)

    _write_sitemapindex_xml(OUTPUT_DIR_PATH / "sitemap.xml", sitemaps)


@backoff.on_exception(backoff.expo, requests.exceptions.RequestException)
def _download_old_data() -> list:
    """Download the currently online data from the server"""
    req = requests.get(OLD_DATA_URL, headers={"Accept-Encoding": "gzip"}, timeout=120)
    if req.status_code != 200:
        logging.warning(f"Could not download online data because of {req.status_code=}. Assuming all are new")
        return []
    old_data = req.json()
    if isinstance(old_data, dict):
        old_data = list(old_data.values())
    return old_data


def _extract_sitemap_data(new_data: list, old_data: list, old_sitemaps: SimplifiedSitemaps) -> Sitemaps:
    """
    Extract sitemap data.

    Lastmod is set to the current time if the entry is modified (indicated via comparing newdata vs olddata),
    or to the last modification time of the online sitemap if the entry is not modified.
    """
    # Each sitemap has a limit of 50MB uncompressed or 50000 entries
    # (that means 1KB per site). We have currently about 33000 entries,
    # so it's unlikely that we'll hit this limit without adding a lot of
    # data. But for the case that a new type of entry is introduced, the
    # sitemap is split into one for rooms and one for the rest.
    sitemaps: Sitemaps = {
        "room": [],
        "other": [],
    }
    old_data_dict = {entry["id"]: entry for entry in old_data}
    new_data_dict = {entry["id"]: entry for entry in new_data}
    changed_count = 0
    for _id, entry in new_data_dict.items():
        sitemap_name: Literal["room"] | Literal["other"] = entry["type"] if entry["type"] in sitemaps else "other"

        # Just copied from the webclient.
        # The webclient doesn't care about the prefix.
        # If the prefix is wrong it'll be corrected (without a redirect).
        # However, this way search engines can already index the final URL.
        url_type_name = {
            "campus": "campus",
            "site": "site",
            "area": "site",
            "building": "building",
            "joined_building": "building",
            "room": "room",
            "virtual_room": "room",
            "poi": "poi",
        }[entry["type"]]
        url = f"https://nav.tum.de/{url_type_name}/{_id}"
        if _id not in old_data_dict or entry != old_data_dict[_id]:
            lastmod = datetime.now(timezone.utc)
            changed_count += 1
        elif old_lastmod := old_sitemaps[sitemap_name].get(url):
            lastmod = old_lastmod
        else:
            lastmod = datetime.now(timezone.utc)
            changed_count += 1

        # Priority is a relative measure from 0.0 to 1.0.
        # The data's `ranking_factors` have arbitrary scaling, but are for
        # rooms in general in the range 0 to 900, so we just add 100, divide by 10_000
        # and clamp to 1.0 for rooms.
        # For buildings etc. that are always >= 10_000, we just subtract 500
        # to get some kind of relative measure.
        if entry["type"] == "room":
            priority = (entry["ranking_factors"]["rank_combined"] + 100) / 10000
        else:
            priority = (entry["ranking_factors"]["rank_combined"] - 500) / 10000
        priority = max(min(priority, 1.0), 0.0)

        sitemaps[sitemap_name].append(
            {
                "url": url,
                "lastmod": lastmod,
                "priority": priority,
            },
        )
    logging.info(f"{changed_count} of {len(new_data) - 1} URLs have been updated.")

    return sitemaps


def _download_online_sitemaps() -> SimplifiedSitemaps:
    """Download online sitemaps by their names"""
    return {
        "room": _download_online_sitemap("https://nav.tum.de/cdn/sitemap-data-room.xml"),
        "other": _download_online_sitemap("https://nav.tum.de/cdn/sitemap-data-other.xml"),
    }


def _download_online_sitemap(url: str) -> dict[str, datetime]:
    """Download a single online sitemap and return a dict of URL -> lastmod time"""
    try:
        req = requests.get(url, headers={"Accept-Encoding": "gzip"}, timeout=10)
    except requests.exceptions.RequestException as error:
        logging.warning(f"Failed to download sitemap '{url}': {error}")
        return {}
    if req.status_code != 200:
        logging.warning(f"Failed to download sitemap '{url}': Status code {req.status_code}")
        return {}

    xmlns = "{http://www.sitemaps.org/schemas/sitemap/0.9}"
    sitemap = {}
    root = defusedET.fromstring(req.text)
    for child in root.iter(f"{xmlns}url"):
        loc = child.find(f"{xmlns}loc")
        lastmod = child.find(f"{xmlns}lastmod")
        if loc is not None and lastmod is not None:
            lastmod_time = datetime.fromisoformat(lastmod.text.rstrip("Z"))
            sitemap[loc.text] = lastmod_time.replace(tzinfo=timezone.utc)
    return sitemap


def _write_sitemap_xml(fname: Path, sitemap: list[SitemapEntry]) -> None:
    """Write the sitemap XML for a single sitemap"""
    urlset = ET.Element("urlset")
    urlset.set("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9")
    for sitemap_entry in sitemap:
        url = ET.SubElement(urlset, "url")
        loc = ET.SubElement(url, "loc")
        loc.text = sitemap_entry["url"]
        lastmod = ET.SubElement(url, "lastmod")
        lastmod.text = sitemap_entry["lastmod"].isoformat(timespec="seconds")
        priority = ET.SubElement(url, "priority")
        priority.text = str(round(sitemap_entry["priority"], 2))

    root = ET.ElementTree(urlset)
    root.write(fname, encoding="utf-8", xml_declaration=True)


def _write_sitemapindex_xml(fname: Path, sitemaps: Sitemaps) -> None:
    """Write the sitemapindex XML"""
    sitemapindex = ET.Element("sitemapindex")
    sitemapindex.set("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9")
    for name, sitemap in sitemaps.items():
        sitemap_el = ET.SubElement(sitemapindex, "sitemap")
        loc = ET.SubElement(sitemap_el, "loc")
        loc.text = f"https://nav.tum.de/cdn/sitemap-data-{name}.xml"
        if lastmod_dates := {site["lastmod"] for site in sitemap if "lastmod" in site}:
            lastmod = ET.SubElement(sitemap_el, "lastmod")
            lastmod.text = max(lastmod_dates).isoformat(timespec="seconds")

    # Because sitemaps cannot be hierarchical, we have to include the
    # webclient sitemap here as well.
    sitemap_el = ET.SubElement(sitemapindex, "sitemap")
    loc = ET.SubElement(sitemap_el, "loc")
    web_sitemap_url = "https://nav.tum.de/sitemap-webclient.xml"
    loc.text = web_sitemap_url
    sitemap = _download_online_sitemap(web_sitemap_url)
    if lastmod_dates := set(sitemap.values()):
        lastmod = ET.SubElement(sitemap_el, "lastmod")
        lastmod.text = max(lastmod_dates).isoformat(timespec="seconds")

    root = ET.ElementTree(sitemapindex)
    root.write(fname, encoding="utf-8", xml_declaration=True)
