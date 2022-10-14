# Server

This folder contains the backend server for NavigaTUM.

## Getting started

### Prerequisites

For getting started, there are some system dependencys which you will need.
Please follow the [system dependencys docs](resources/documentation/Dependencys.md) before trying to run this part of our project.

### Get the data

The data is provided to the server with just a simple JSON file.
You can create a `data` subdirectory and copy the `api_data.json`
(and optionally `search_data.json`) file into it.

Alternatively, link the output directory to the server data directory,
so that you don't need to copy on every update:

```bash
ln -s ../data/output data
```

### 2. Starting the server

Run

```bash
cargo run --release
```

The server should now be available on `localhost:8080`.

### Setup MeiliSearch (optional)

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
curl -X PUT 'http://localhost:7700/indexes/entries/settings/filterable-attributes' --data '["facet"]'

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

If you have made changes to the API, you need to update the API documentation.

There are two editors for the API documentation (both are imperfect):

- [Swagger Editor](https://editor.swagger.io/?url=https://raw.githubusercontent.com/TUM-Dev/navigatum/main/openapi.yaml)
- [stoplight](stoplight.io)

Of course documentation is one part of the process. If the changes are substantial, you should also run an API-Fuzz-Test:
To make sure that this specification is up-to-date and without holes, we run [schemathesis](https://github.com/schemathesis/schemathesis) using the following command on API Server:

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
