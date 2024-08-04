# Data-Pipeline

This folder contains:

- The code to compile the datasets for NavigaTUM
- Custom data inserted into the datasets
- Custom patches applied on the source data

The code to retrieve external data, as well as externally retrieved data is located under `external`.

> [!WARNING]
> A lot of this code is more a work-in-progress than finished.
> Especially features such as POIs, custom maps or other data types such as events are drafted but not yet fully implemented.
>
> New external data might break the scripts from time to time,
> - as either rooms or buildings are removed,
> - the external data has errors,
> - or we make assumptions here that turn out to be wrong.

## Getting started

### Prerequisites

For getting started, there are some system dependencies which you will need.
Please follow the [system dependencies docs](/resources/documentation/Dependencys.md) before trying to run this part of
our project.

### Dependencies

Since data needs some python dependency's, you will need to install them first.
We recommend doing this in a virtual environment.

From the root of the project, run:

```bash
python3 -m venv venv
source venv/bin/activate
pip install -r data/requirements.txt -r requirements-dev.txt
```

## Getting external data

> [!TIP]
> The latest scraped data is already included in the `external/results` directory,
> you do not need to run the scraping yourself and can skip to the next step

External data (and the scrapers) are stored in the `external/` subdirectory.

You can run a scraper from `external/scraper/`.
All newer scrapers are pretty quick => no shenanigans like commenting out lines are needed.

You can scrape with:

```bash
cd external
export PYTHONPATH=$PYTHONPATH:..
python3 nat.py
python3 public_transport.py
python3 roomfinder.py
export CONNECTUM_OAUTH_CLIENT_ID=GIVEN_OUT_AS_NEEDED
export CONNECTUM_OAUTH_CLIENT_SECRET=GIVEN_OUT_AS_NEEDED
python3 tumonline.py
```

### Compiling the data

```bash
python3 compile.py
```

The exported datasets will be stored in `output/`
as [JSON](https://www.json.org/json-de.html)/[Parquet](https://wikipedia.org/wiki/Apache_Parquet) files.

### Directory structure

```bash
data
├── external/
│   ├── output/   # 🠔 Here the final, compiled datasets will be stored
│   └── scrapers/ # how we download
├── processors/   # Processing code
├── sources/      # Custom data and patches
│   ├── img/
│   └── <custom data>
└── compile.py           # The main script to compile the datasources into our data representation
```

Deployment related there are also these files:

```bash
data
├── Dockerfile       # Main dockerfile, in the deployment this is sometimes called the cdn
├── ngnix.conf       # nginx configuration file used by above Dockerfile
└── requirements.txt # python dependency's
```

### How the data looks like

```json
{
  "entry-id": {
    "id": "entry-id",
    "type": "room",
    ...
    data
    as
    specified
    in
    `
    data-format.yaml
    `
  },
  ...
  all
  other
  entries
  in
  the
  same
  form
}
```

## Compilation process

The data compilation is made of indiviual processing steps, where each step adds new or modifies the current data. The
basic structure of the data however stays the same from the beginning on and is specified in `data-format_*.yaml`.

- **Step 00**: The first step reads the base root node, areas, buildings etc. from the
  `sources/00_areatree` file and creates an object collection (python dictionary)
  with the data format as mentioned above.
- **Steps 01-29**: Within these steps, new rooms or POIs might be added, however no
  new areas or buildings, since all areas and buildings have to be defined in the
  _areatree_. After them, no new entries are being added to the data.
    - **Steps 0x**: Supplement the base data with extended custom data.
    - **Steps 1x**: Import rooms and building information from external sources
    - **Steps 2x**: Import POIs
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
It (currently) has a custom data format to be human & machine-readable while taking only minimal space.
Details about the formatting are given at the head of the file.

## License

The source data (i.e. all files located in `sources/` that are not images) is made available under the Open Database
License: <https://opendatacommons.org/licenses/odbl/1.0/>.
Any rights in individual contents of the database are licensed under the Database Contents
License: <http://opendatacommons.org/licenses/dbcl/1.0/>.

> [!WARNING]
> The images in `sources/img/` are subject to their own licensing terms, which are stated in the
> file `sources/img/img-sources.yaml`.
> The compiled database may contain contents from external sources (i.e. all files in `external/`) that do have
> different license terms.

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
