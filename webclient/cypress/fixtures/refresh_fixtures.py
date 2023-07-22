import json
import urllib.request
import requests
from pathlib import Path

FIXTURES = Path(__file__).parent


def to_filepath(url_path: str) -> Path:
  """Converts an url path to a filepath"""
  filename = f"{urllib.request.url2pathname(url_path)}.json"
  filepath = FIXTURES / filename.removeprefix("/")
  filepath.parent.mkdir(parents=True, exist_ok=True)
  return filepath


def scrape(url_path: str) -> None:
  """Scrapes the given url path and saves it to a file"""
  req = requests.get(f"https://nav.tum.de/api{url_path}", timeout=10)
  filepath = to_filepath(url_path)
  with open(filepath, "w", encoding="utf-8") as file:
    data = json.loads(req.text)
    json.dump(data, file, sort_keys=True, indent=2)
  print(url_path.ljust(20), "->", filepath)


if __name__ == "__main__":
  # details
  for item in ["root", "mi", "mw", "5502.U1.234M"]:
    scrape(f"/get/{item}")
  # search
  for full_search in ["fsmw", "fsmpic"]:
    for until_index in range(len(full_search)):
      scrape(f"/search?q={full_search[:until_index + 1]}")
