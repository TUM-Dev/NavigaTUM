# NavigaTUM Data Repository

This repository contains:

- The code to compile the datasets for NavigaTUM
- Custom data inserted into the datasets
- Custom patches applied on the source data

The code to retrieve external data, as well as externally retrieved data is not part of this repository.

⚠️ A lot of this code is more a work in progress than finished. Especially features such as POIs, custom maps or other data types such as events are drafted but not yet fully implemented.

Also, new external data might break the scripts from time to time, as either rooms or buildings are removed, the external data has errors or we make assumptions here that turn out to be wrong.

## Getting started

### Dependencies

You need the following dependencies to get started:

- _Python_ (at least version 3.6)
- The following Python packages:
  `pip install -r requirements.txt`

## Getting external data

External data (and the scraper) is stored in the `external/` subdirectory.

By default, the latest scraped data is already included in this directory, so that you do not need
to run the scraping yourself and skip to the next step.

However if you want to update the scraped data, open `external/main.py` and comment out all
steps depending on what specific data you want to scrape (Note that some steps depend on previous
steps. In this case, the downloader will automatically run these as well).

Then, start scraping with:

```bash
cd external
python main.py
```

The data will be stored in the `cache` subdirectory as json files. To force a redownload, delete them.

As a last step, move the `.json` files from the cache directory into the external directory, so that
it contains the most recent scraped results, and then go back:

```bash
mv cache/buildings* cache/rooms* cache/maps* cache/usages* .
cd ..
```

### Compiling the data

```bash
python compile.py
```

The exported datasets will be stored in `output/` as JSON files.

### Directory structure

```bash
data
├── external/    # 🠔 This is the sub-repository containing externally retrieved data
├── output/      # 🠔 Here the final, compiled datasets will be stored
├── processors/  # 🠔 Processing code
├── sources/     # 🠔 Custom data and patches
│   ├── img/
│   └── <custom data>
├── compile.py           # 🠔 The main script
├── data-format_*.yaml   # 🠔 Data format specification
└── search_synonyms.json # 🠔 synonyms that MeiliSearch considers
```

Deployment related there are also these files:

```bash
data
├── Dockerfile # 🠔 Main dockerfile, in the deployment this is sometimes called the cdn
├── ngnix.conf # 🠔 ngnix cofigureation file used by above Dockerfile
└── requirements.txt # 🠔 python dependencys
```

### How the data looks like

```json
{
    "entry-id": {
        "id": "entry-id",
        "type": "room",
        ... data as specified in `data-format.yaml`
    },
    ... all other entries in the same form
}
```

## Compilation process

The data compilation is made of indiviual processing steps, where each step adds new or modifies the current data. The basic structure of the data however stays the same from the beginning on and is specified in `data-format_*.yaml`.

- **Step 00**: The first step reads the base root node, areas, buildings etc. from the
  `sources/00_areatree` file and creates an object collection (python dictionary)
  with the data format as mentioned above.
- **Steps 01-29**: Within these steps, new rooms or POIs might be added, however no
  new areas or buildings, since all areas and buildings have to be defined in the
  _areatree_. After them, no new entries are being added to the data.
  - **Steps 0x**: Supplement the base data with extended custom data.
  - **Steps 1x**: Import rooms and building information from external sources
  - **Steps 2x**: -
- **Steps 30-89**: Later steps are intended to augment the entries with even more
  information and to ensure a consistent format. After them, no new (external or custom)
  information should be added to the data.
  - **Steps 3x**: Make data more coherent & structural stuff
  - **Steps 4x**: Coordinates and maps
  - **Steps 5x**: Add images
  - **Steps 6x**: -
  - **Steps 7x**: -
  - **Steps 8x**: Generate properties and sections (such as overview sections)
- **Steps 90-99**: Process and export for search.
- **Step 100**: Export final data (for use in the API). Some temporary data fields might be removed at this point.

### Details

#### Step 00 Areatree

The starting point is the data defined in the "areatree" (in `sources/00_areatree`).
It (currently) has a custom data format to be human & machine-readable while taking
only minimal space.
Details about the formatting are given at the head of the file.

## License

The source data (i.e. all files located in `sources/` that are not images) is made available under the Open Database License: <http://opendatacommons.org/licenses/odbl/1.0/>. Any rights in individual contents of the database are licensed under the Database Contents License: <http://opendatacommons.org/licenses/dbcl/1.0/>.

The images in `sources/img/` are subject to their own licensing terms, which are stated in the file `sources/img/img-sources.yaml`.

_Please note that the compiled database may contain contents from external sources (i.e. all files in `external`) that do have different license terms._

---

The Python code is distributed under the GNU GPL v3:

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
