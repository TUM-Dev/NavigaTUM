# Server

This folder contains the main backend server for NavigaTUM.

## Getting started

### Prerequisites

For getting started, there are some system dependencies which you will need.
Please follow the [system dependencies docs](/SYSTEM_DEPENDENCIES.md
) before trying to run this part of
our project.

### Additional dependency

We have a few API endpoints which require additional dependencies.

As a general rule of thumb, need to **do the Database** and **MeiliSearch** setup.

The `preview` endpoint is the only endpoint, which requires the tileserver.
Because of the data required for download and how non-essential this part is, it is only provided via the production
instance.

#### How to Set up the Databases

At the beginning of the main API we set up both meilisearch and the database.
This will ensure that the sqlite database and meilisearch index is created.

This requires meilisearch to be online.
To set up [MeiliSearch](https://github.com/meilisearch/MeiliSearch), either follow their installation instructions or
use

```bash
docker run -it --rm -p 7700:7700 getmeili/meilisearch:latest
```

MeiliSearch provides an interactive interface at <http://localhost:7700>.

To set up the Postgis, run the following command:

```bash
docker run -it --rm -e POSTGRES_PASSWORD=CHANGE_ME -p 5432:5432 postgis/postgis:latest
```

### Starting the server

Run `cargo run` to start the server.
The server should now be available on `localhost:8080` if you have configured the correct environment.

> [!NOTE]
> `cargo run --release` is used to start the server for an optimised production build (use this if you want to profile
> the `search` or `preview` functions, it makes quite a difference).

### Static Files and CDN

The server now serves all static files (images, maps, sitemaps, data files) directly at the `/cdn` endpoint, eliminating the need for a separate nginx CDN container.

#### Static Data Files

The server can load setup data files from the local filesystem or download them from the CDN. This provides:
- Faster startup times in production (all files are baked into the Docker image)
- Self-contained deployment (no separate CDN container needed)
- Using local files during development, if available
- Fallback to downloading files during development when local files aren't available

#### File Locations

The server looks for data files in the following locations (in order):
1. `/cdn/` (Docker production - symlinked to `/app/cdn/`)
2. `data/output/` (relative to current working directory)
3. `../data/output/` (one level up - useful when running from `server/` directory)
4. `../../data/output/` (two levels up)

If files are not found locally, they will be downloaded from the source specified by the `CDN_URL` environment variable (or GitHub for some files).

#### Required Files

The following files are loaded during server setup:
- `alias_data.parquet` - Alias mappings for locations (baked into Docker image)
- `api_data.json` - Main location data for the API (baked into Docker image)
- `status_data.parquet` - Status information for locations (baked into Docker image)
- `search_data.json` - Search index data for MeiliSearch (baked into Docker image)
- `public_transport.parquet` - Public transportation station data (baked into Docker image)

### Environment Variables

| variable                          | module                           |                                         | usage/description                                                                                      |
|-----------------------------------|----------------------------------|-----------------------------------------|--------------------------------------------------------------------------------------------------------|
| `POSTGRES_{USER,PASSWORD,URL,DB}` | [`all`](./main.rs)               | required                                | Used to connect to the db                                                                              |
| `GIT_COMMIT_SHA`                  | [`main`](./main.rs)              | optional                                | Shown in the status endpint (also set at build time in docker)                                         |
| `LOG_LEVEL`                       | [`main`](./main.rs)              | optional                                | Controlls what is being logged (default=`info` in release and `debug` in development mode)             |
| `GITHUB_TOKEN`                    | [`feedback`](./feeedback/mod.rs) |                                         | A GitHub token with `write` access to `repo`.<br/>This is used to create issues/PRs on the repository. |
| `JWT_KEY`                         | [`feedback`](./feeedback/mod.rs) |                                         | A key used to sign JWTs.<br/>This is used to authenticate that feedback tokens were given out by us.   |
| `MIELI_{URL,MASTER_KEY}`          | [`search`](./search/mod.rs)      |                                         | Allows searching via meiliserch                                                                        |
| `CDN_URL`                         | [`setup`](./setup/mod.rs)        | optional (fallback only)                | Fallback URL for downloading data files if not found locally (usually not needed in production)        |

### Adding Migrations

For the database-connector we use sqlx.
Migrations can be run with the `sqlx-cli` tool. Said tool can be installed with:

```bash
cargo install sqlx-cli
```

Migrations can be added using

```bash
cargo sqlx migrate add -r <migration-name>
```

### Adding/editing database queries

To get compiletime guarantees for our queries, we use sqlx.
To add/edit a query, you will need to run the following command:

```bash
cargo sqlx migrate run --database-url postgres://postgres:CHANGE_ME@localhost:5432/postgres
cargo sqlx prepare --database-url postgres://postgres:CHANGE_ME@localhost:5432/postgres
```

### API-Changes

#### Editing

If you have made changes to the API, you need to update the API documentation.

There are two editors for the API documentation (both are imperfect):

- [Swagger Editor](https://editor.swagger.io/?url=https://nav.tum.de/api/openapi.json)
- [stoplight](https://stoplight.io/)

#### Testing

Of course documentation is one part of the process.
Run API-Fuzz-Test and [schemathesis](https://github.com/schemathesis/schemathesis) on API Server to ensure specification
is up-to-date and without holes.
To do so, run the following commands against the API Server:

```bash
python -m venv venv
source venv/bin/activate
pip install schemathesis
st run --workers=auto --base-url=http://localhost:3003 --checks=all https://nav.tum.de/api/openapi.json
```

Some fuzzing-goals may not be available for you locally, as they require prefix-routing (f.ex.`/cdn` to the CDN) and
some fuzzing-goals are automatically tested in our CI.  
You can exchange `--base-url=http://localhost:3003` to `--base-url=https://nav.tum.de` for the full public API, or
restrict your scope using an option like `--endpoint=/api/search`.

### Approval tests

Some of our tests are approval tests.
Please install [insta](https://insta.rs/docs/quickstart/) to have a working environment.

You can then run `cargo insta test` instead of `cargo test` to review the needed changes.
If you don't want to do this, using the version we provide via CI is fine, but the DX is way better with the correct
tooling.

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
