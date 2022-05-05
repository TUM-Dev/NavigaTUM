import json
import urllib.request
import xml.etree.ElementTree as ET
import defusedxml.ElementTree as defusedET  # supports only parse()
from datetime import datetime


def generate_sitemap():
    """ Generate a sitemap that diffs changes since to the currently online data """
    # Load exported data. This function is intentionally not using the data object
    # directly, but re-parsing the output file instead, because the export not
    # export all fields. This way we're also guaranteed to have the same types
    # (and not e.g. numpy floats).
    with open("output/api_data.json") as f:
        new_data = json.load(f)

    # Currently online data
    req = urllib.request.Request("https://nav.tum.sexy/cdn/api_data.json")
    with urllib.request.urlopen(req) as resp:
        old_data = json.loads(resp.read().decode("utf-8"))

    # Each sitemap has a limit of 50MB uncompressed or 50000 entries
    # (that means 1KB per site). We have currently about 33000 entries,
    # so it's unlikely that we'll hit this limit without adding a lot of
    # data. But for the case that a new type of entry is introduced, the
    # sitemap is split into one for rooms and one for the rest.
    # Note that the root element is included, which will link to the main page.
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
        sitemap_name = entry["type"] if entry["type"] in sitemaps else "other"

        url = f"https://nav.tum.sexy/view/{_id}"

        if _id not in old_data or entry != old_data[_id]:
            last_changed = datetime.now()
            changed_count += 1
        else:
            # Try to look up the last changed date in the old sitemap
            last_changed = old_sitemaps.get(sitemap_name, {})\
                                       .get(url, datetime.now())

        # Priority is a relative measure from 0.0 to 1.0.
        # The data's `ranking_factors` have arbitrary scaling, but are for
        # rooms in general in the range 0 to 9, so we just add 1, divide by 10
        # and clamp to 1.0 for rooms.
        # For buildings etc. that are always >= 10, we just subtract 5
        # to get some kind of relative measure. Also for root (which is
        # downrated in search) we set 1.0 by hand.
        if entry["type"] == "root":
            priority = 1.0
        elif entry["type"] == "room":
            priority = min((entry["ranking_factors"]["rank_combined"] + 1) / 10, 1.0)
        else:
            priority = min((entry["ranking_factors"]["rank_combined"] - 5) / 10, 1.0)

        sitemaps[sitemap_name].append({
            "url": url,
            "last_changed": last_changed,
            "priority": priority,
        })

    print(f"{changed_count} of {len(new_data)} URLs have been updated.")

    for name, sitemap in sitemaps.items():
        _write_sitemap_xml(f"output/sitemap-data-{name}.xml", sitemap)

    _write_sitemapindex_xml(f"output/sitemap.xml", sitemaps)


def _download_online_sitemaps(sitemap_names):
    """ Download online sitemaps by their names """
    xmlns = "{http://www.sitemaps.org/schemas/sitemap/0.9}"
    sitemaps = {}
    for name in sitemap_names:
        req = urllib.request.Request(f"https://nav.tum.sexy/sitemap-data-{name}.xml")
        try:
            with urllib.request.urlopen(req) as resp:
                sitemap_str = resp.read().decode("utf-8")
                sitemaps[name] = {}
                root = defusedET.fromstring(sitemap_str)
                for child in root.iter(f"{xmlns}url"):
                    loc = child.find(f"{xmlns}loc")
                    lastmod = child.find(f"{xmlns}lastmod")
                    if loc is not None and lastmod is not None:
                        sitemaps[name][loc.text] = datetime.fromisoformat(lastmod.text)
        # We gracefully catch any problems (e.g. a 404 HTTPError), because
        # having the old sitemaps is not critical.
        except Exception as e:
            print(f"Failed to download sitemap '{name}': {e}")

    return sitemaps


def _write_sitemap_xml(fname, sitemap):
    """ Write the sitemap XML for a single sitemap """
    urlset = ET.Element("urlset")
    urlset.set("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9")
    for sitemap_entry in sitemap:
        url = ET.SubElement(urlset, "url")
        loc = ET.SubElement(url, "loc")
        loc.text = sitemap_entry["url"]
        lastmod = ET.SubElement(url, "lastmod")
        lastmod.text = sitemap_entry["last_changed"].isoformat()
        priority = ET.SubElement(url, "priority")
        priority.text = str(round(sitemap_entry["priority"], 2))

    root = ET.ElementTree(urlset)
    root.write(fname, encoding="utf-8", xml_declaration=True)


def _write_sitemapindex_xml(fname, sitemaps):
    """ Write the sitemapindex XML """
    sitemapindex = ET.Element("sitemapindex")
    sitemapindex.set("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9")
    for name in sitemaps.keys():
        sitemap_el = ET.SubElement(sitemapindex, "sitemap")
        loc = ET.SubElement(sitemap_el, "loc")
        loc.text = f"https://nav.tum.sexy/sitemap-data-{name}.xml"
        changefreq = ET.SubElement(sitemap_el, "changefreq")
        changefreq.text = "daily"  # data sitemaps might change frequently

    # Because sitemaps cannot be hierarchical, we have to include the
    # webclient sitemap here as well.
    sitemap_el = ET.SubElement(sitemapindex, "sitemap")
    loc = ET.SubElement(sitemap_el, "loc")
    loc.text = f"https://nav.tum.sexy/sitemap-webclient.xml"
    changefreq = ET.SubElement(sitemap_el, "changefreq")
    # webclient `about` pages are important, but don't change frequently
    changefreq.text = "weekly"

    root = ET.ElementTree(sitemapindex)
    root.write(fname, encoding="utf-8", xml_declaration=True)
