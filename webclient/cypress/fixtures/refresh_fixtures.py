import json
import re
from pathlib import Path
import requests

FIXTURES=Path(__file__).parent

def to_file_name(key: str) -> str:
    """"""
    key = re.sub("[^a-zA-Z0-9&?]", " ", key)
    key = re.sub("([A-Z]+)", r" \1", key)
    key = re.sub("([A-Z][a-z]+)", r" \1", key)
    return "_".join(key.split()).lower()

def scrape(url_path:str) -> None:
    req = requests.get(f"http://localhost:8080/api{url_path}", timeout=10)
    filename = f"{to_file_name(url_path)}.json"
    with open(FIXTURES/filename, "w", encoding="utf-8") as file:
        data=json.loads(req.text)
        json.dump(data, file,sort_keys=True,indent=2)
    print(url_path.ljust(20), "->", filename)

if __name__ == "__main__":
    # details
    for item in ["root", "mi", "mw", "5502.U1.234M"]:
      scrape(f"/get/{item}")
    # search
    for full_search in ["fsmw", "fsmpic"]:
        for until_index in range(len(full_search)):
            scrape(f"/search?q={full_search[:until_index+1]}")
