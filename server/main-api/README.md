# Main-API

This folder contains the main backend server for NavigaTUM.

## Getting started

### Prerequisites

For getting started, there are some system dependencys which you will need.
Please follow the [system dependencys docs](/resources/documentation/Dependencys.md) before trying to run this part of our project.

### Starting the server

Run `cargo run` to start the server.
The server should now be available on `localhost:8080`.

Note that `cargo run --release` is used to start the server for an optimised production build (use this if you want to profile the `search` or `preview` functions, it makes quite a difference).

### Additional dependency's for some API endpoints

We have a few API endpoints which require additional dependencies.

As a general rule of thumb, if you probably want to **skip the tileserver**, but want to **do the SQLite Database** and **MeiliSearch** setup.
The reason for this is, that the `preview` endpoint is the only endpoint, which requires the tileserver and said endpoint is a non-essential part of the project.

#### How to Set up the Sqlite Database (needed for the `get` and `preview` endpoints)

##### Getting the data

To populate the database, you will need to get said data.
There are multiple ways to do this, but the easiest way is to download the data from our [website](https://nav.tum.de/).

(Assuming you are in the `server` directory)

```bash
mkdir -p data
wget -P data https://nav.tum.de/cdn/api_data.json
```

##### Setting up the database

To set up the database, you will need to run the `load_api_data_to_db.py` script:

```bash
python3 load_api_data_to_db.py
```

#### How to Set up the tileserver (needed for the `preview` endpoint)

To set up your tileserver, head over to the [`map`](https://github.com/TUM-Dev/NavigaTUM/tree/main/map) folder and follow the instructions there.

#### How to Set up MeiliSearch (needed for the `search` endpoint)

The server uses [MeiliSearch](https://github.com/meilisearch/MeiliSearch) as a backend for search.
For a local test environment you can skip this step if you don't want to test or work on search.

There are a lot of different ways to run MeiliSearch (see on their repo). Here we compile it
from sources:

```bash
# Clone MeiliSearch
cd ..
git clone https://github.com/meilisearch/MeiliSearch.git -b v0.22.0
cd MeiliSearch

# Build and run
cargo run --release
```

Next, we need to add our index and configure search:

```bash
# Create index
curl -i -X POST 'http://localhost:7700/indexes' --header 'content-type: application/json' --data '{ "uid": "entries", "primaryKey": "ms_id" }'

# Set filterable attributes
curl -X PUT 'http://localhost:7700/indexes/entries/settings/filterable-attributes' --data '["facet", "parent_keywords", "parent_building_names", "campus", "type", "usage"]'

# Upload entries data
curl -i -X PUT 'http://localhost:7700/indexes/entries/documents' --header 'content-type: application/json' --data-binary @data/search_data.json

# Configure index
curl -X PUT 'http://localhost:7700/indexes/entries/settings/ranking-rules' --data '["words","typo","rank:desc","exactness","proximity","attribute"]'

curl -X PUT 'http://localhost:7700/indexes/entries/settings/synonyms' --data @../data/search_synonyms.json

curl -X PUT 'http://localhost:7700/indexes/entries/settings/searchable-attributes' --data '[ "ms_id", "name", "arch_name", "type", "type_common_name", "parent_building", "parent_keywords", "address", "usage" ]'
```

If you want to update the data in the index, run:

```bash
curl -i -X PUT 'http://localhost:7700/indexes/entries/documents' --header 'content-type: application/json' --data-binary @data/search_data.json
```

And if you want to delete the index, run:

```bash
curl -X DELETE 'http://localhost:7700/indexes/entries'
```

MeiliSearch provides an interactive interface at [http://localhost:7700](http://localhost:7700).

### API-Changes

#### Editing

If you have made changes to the API, you need to update the API documentation.

There are two editors for the API documentation (both are imperfect):

- [Swagger Editor](https://editor.swagger.io/?url=https://raw.githubusercontent.com/TUM-Dev/navigatum/main/openapi.yaml)
- [stoplight](https://stoplight.io/)

#### Testing

Of course documentation is one part of the process.
Run API-Fuzz-Test and [schemathesis](https://github.com/schemathesis/schemathesis) on API Server to ensure specification is up-to-date and without holes.
To do so, run the following commands against the API Server:

```bash
python -m venv venv
source venv/bin/activate
pip install schemathesis
st run --workers=auto --base-url=http://localhost:8080 --checks=all ../openapi.yaml
```

Some fuzzing-goals may not be available for you locally, as they require prefix-routing (f.ex.`/cdn` to the CDN) and some fuzzing-goals are automatically tested in our CI.  
You can exchange `--base-url=http://localhost:8080` to `--base-url=https://nav.tum.sexy` for the full public API, or restrict your scope using a option like `--endpoint=/api/search`.

## License

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

---
