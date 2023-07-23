import json
import urllib.request
import requests
from pathlib import Path

FIXTURES = Path(__file__).parent


def to_filepath(filename: str) -> Path:
  """Converts an url path to a filepath"""
  filepath = FIXTURES / filename.removeprefix("/")
  filepath.parent.mkdir(parents=True, exist_ok=True)
  return filepath



def scrape(url_path: str, filename) -> None:
  """Scrapes the given url path and saves it to a file"""
  req = requests.get(f"https://nav.tum.de/api{url_path}", timeout=10)
  filepath = to_filepath(filename)
  with open(filepath, "w", encoding="utf-8") as file:
    data = json.loads(req.text)
    json.dump(data, file, sort_keys=True, indent=2)
  relative_path = str(filepath.relative_to(FIXTURES))
  print(f"{relative_path.ljust(40)} <- {url_path}")


if __name__ == "__main__":
  print("--- details endpoint ---")
  for item in ["root", "mi", "mw", "5502.U1.234M", "garching-interims", "garching"]:
    scrape(f"/get/{item}?lang=de", f"get/{item}.de.json")

  print("--- seach endpoint ---")
  for full_search in ["fsmb", "fsmpic"]:
    for until_index in range(len(full_search)):
      query=full_search[:until_index + 1]
      scrape(f"/search?q={query}&lang=de", f"search/{query}.de.json")
      scrape(f"/search?q={query}&limit_buildings=10&limit_rooms=30&limit_all=30&lang=de", f"search/{query}.long.de.json")
