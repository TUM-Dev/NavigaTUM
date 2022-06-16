import json
import logging
import urllib.error
import urllib.request
import xml.etree.ElementTree as ET  # nosec: used for writing to a file, not for reading
from datetime import datetime

# supports only parse()
from defusedxml import ElementTree as defusedET  # type:ignore


def generate_sitemap():
    """Generate a sitemap that diffs changes since to the currently online data"""
    # Load exported data. This function is intentionally not using the data object
    # directly, but re-parsing the output file instead, because the export not
    # export all fields. This way we're also guaranteed to have the same types
    # (and not e.g. numpy floats).
    with open("output/api_data.json", encoding="utf-8") as file:
        new_data = json.load(file)

    # Currently online data
    req = urllib.request.Request("https://nav.tum.sexy/cdn/api_data.json")
    with urllib.request.urlopen(req) as resp:  # nosec: url parameter is fixed and does not allow for file traversal
        old_data = json.loads(resp.read().decode("utf-8"))

    # Each sitemap has a limit of 50MB uncompressed or 50000 entries
    # (that means 1KB per site). We have currently about 33000 entries,
    # so it's unlikely that we'll hit this limit without adding a lot of
    # data. But for the case that a new type of entry is introduced, the
    # sitemap is split into one for rooms and one for the rest.
    # Note that the root element is not included, because it just redirects
    # to the main page.
    sitemaps = {
        "room": [],
        "other": [],
    }

    # Look whether there are currently online sitemaps for the provided
    # sitemaps name. In case there aren't, we assume this sitemap is new,
    # and all entries will be marked as changed.
    old_sitemaps = _download_online_sitemaps(sitemaps.keys())

    changed_count = 0
    for _id, entry in new_data.items():
        if entry["type"] == "root":
            continue

        sitemap_name = entry["type"] if entry["type"] in sitemaps else "other"

        # Just copied from the webclient. The webclient doesn't care about
        # the prefix – if it is wrong it'll be corrected (without a redirect).
        # However this way search engines can already index the final URL.
        url_type_name = {
            "campus": "campus",
            "site": "site",
            "area": "site",
            "building": "building",
            "joined_building": "building",
            "room": "room",
            "virtual_room": "room",
        }[entry["type"]]
        url = f"https://nav.tum.sexy/{url_type_name}/{_id}"
        if _id not in old_data or entry != old_data[_id]:
            lastmod = datetime.utcnow()
            changed_count += 1
        else:
            # Try to look up the last changed date in the old sitemap
            lastmod = old_sitemaps.get(sitemap_name, {}).get(url, None)
            if lastmod is None:
                lastmod = datetime.utcnow()
                changed_count += 1

        # Priority is a relative measure from 0.0 to 1.0.
        # The data's `ranking_factors` have arbitrary scaling, but are for
        # rooms in general in the range 0 to 900, so we just add 100, divide by 10_000
        # and clamp to 1.0 for rooms.
        # For buildings etc. that are always >= 10_000, we just subtract 500
        # to get some kind of relative measure.
        if entry["type"] == "room":
            priority = min((entry["ranking_factors"]["rank_combined"] + 100) / 10000, 1.0)
        else:
            priority = min((entry["ranking_factors"]["rank_combined"] - 500) / 10000, 1.0)

        sitemaps[sitemap_name].append(
            {
                "url": url,
                "lastmod": lastmod,
                "priority": priority,
            },
        )

    logging.info(f"{changed_count} of {len(new_data) - 1} URLs have been updated.")

    for name, sitemap in sitemaps.items():
        _write_sitemap_xml(f"output/sitemap-data-{name}.xml", sitemap)

    _write_sitemapindex_xml("output/sitemap.xml", sitemaps)


def _download_online_sitemaps(sitemap_names):
    """Download online sitemaps by their names"""
    sitemaps = {}
    for name in sitemap_names:
        sitemaps[name] = _download_online_sitemap(f"https://nav.tum.sexy/cdn/sitemap-data-{name}.xml")
    return sitemaps


def _download_online_sitemap(url):
    xmlns = "{http://www.sitemaps.org/schemas/sitemap/0.9}"  # noqa: FS003
    req = urllib.request.Request(url)
    try:
        with urllib.request.urlopen(req) as resp:  # nosec: url parameter is fixed and does not allow for file traversal
            sitemap_str = resp.read().decode("utf-8")
            sitemap = {}
            root = defusedET.fromstring(sitemap_str)
            for child in root.iter(f"{xmlns}url"):
                loc = child.find(f"{xmlns}loc")
                lastmod = child.find(f"{xmlns}lastmod")
                if loc is not None and lastmod is not None:
                    sitemap[loc.text] = datetime.fromisoformat(lastmod.text.rstrip("Z"))
    except urllib.error.HTTPError as error:
        logging.warning(f"Failed to download sitemap '{url}': {error}")
    return sitemap


def _write_sitemap_xml(fname, sitemap):
    """Write the sitemap XML for a single sitemap"""
    urlset = ET.Element("urlset")
    urlset.set("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9")
    for sitemap_entry in sitemap:
        url = ET.SubElement(urlset, "url")
        loc = ET.SubElement(url, "loc")
        loc.text = sitemap_entry["url"]
        lastmod = ET.SubElement(url, "lastmod")
        lastmod.text = sitemap_entry["lastmod"].isoformat(timespec="seconds") + "Z"
        priority = ET.SubElement(url, "priority")
        priority.text = str(round(sitemap_entry["priority"], 2))

    root = ET.ElementTree(urlset)
    root.write(fname, encoding="utf-8", xml_declaration=True)


def _write_sitemapindex_xml(fname, sitemaps):
    """Write the sitemapindex XML"""
    sitemapindex = ET.Element("sitemapindex")
    sitemapindex.set("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9")
    for name, sitemap in sitemaps.items():
        sitemap_el = ET.SubElement(sitemapindex, "sitemap")
        loc = ET.SubElement(sitemap_el, "loc")
        loc.text = f"https://nav.tum.sexy/cdn/sitemap-data-{name}.xml"
        # we set the lastmod to the latest lastmod of all sitemaps
        lastmod_dates = {site["lastmod"] for site in sitemap if "lastmod" in site}
        if lastmod_dates:
            lastmod = ET.SubElement(sitemap_el, "lastmod")
            lastmod.text = max(lastmod_dates).isoformat(timespec="seconds") + "Z"

    # Because sitemaps cannot be hierarchical, we have to include the
    # webclient sitemap here as well.
    sitemap_el = ET.SubElement(sitemapindex, "sitemap")
    loc = ET.SubElement(sitemap_el, "loc")
    web_sitemap_url = "https://nav.tum.sexy/sitemap-webclient.xml"
    loc.text = web_sitemap_url
    sitemap = _download_online_sitemap(web_sitemap_url)
    lastmod_dates = set(sitemap.values())
    if lastmod_dates:
        lastmod = ET.SubElement(sitemap_el, "lastmod")
        lastmod.text = max(lastmod_dates).isoformat(timespec="seconds") + "Z"

    root = ET.ElementTree(sitemapindex)
    root.write(fname, encoding="utf-8", xml_declaration=True)
