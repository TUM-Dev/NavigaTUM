# Calendar

This folder contains the calendar-microservice for NavigaTUM.

## Getting started

### Prerequisites

For getting started, there are some system dependencys which you will need.
Please follow the [system dependencys docs](resources/documentation/Dependencys.md) before trying to run this part of our project.

### How to Set up the Sqlite Database

#### Setting up the database

To set up the database, you will need to run a `postgres` instance. We recommend this configuration:

```bash
docker run -p 5432:5432 -e POSTGRES_PASSWORD=password postgres
```

Any migrations this database needs are applied on first run of the server.
In order to change the connection parameters that the calendar server uses you can use the environment variables:

- `POSTGRES_USER` (default `postgres`)
- `POSTGRES_PASSWORD` (default `password`)
- `POSTGRES_URL` (default `localhost`, specify the port if different from 5432)
- `POSTGRES_DB` (default: same as `POSTGRES_USER`)

### Starting the server

Run `cargo run` to start the server.
The server should now be available on `localhost:8081`.

Note that `cargo run --release` is used to start the server for an optimised production build (use this if you want to profile performance, it makes quite a difference).

### API-Changes

#### Editing

If you have made changes to the API, you need to update the API documentation.

There are two editors for the API documentation (both are imperfect):

- [Swagger Editor](https://editor.swagger.io/?url=https://raw.githubusercontent.com/TUM-Dev/navigatum/main/openapi.yaml)
- [stoplight](stoplight.io)

#### Testing

Of course documentation is one part of the process. If the changes are substantial, you should also run an API-Fuzz-Test:
To make sure that this specification is up-to-date and without holes, we run [schemathesis](https://github.com/schemathesis/schemathesis) using the following command on API Server:

```bash
python -m venv venv
source venv/bin/activate
pip install schemathesis
st run --workers=auto --base-url=http://localhost:8060 --checks=all ../openapi.yaml
```

Some fuzzing-goals may not be available for you locally, as they require prefix-routing (f.ex.`/cdn` to the CDN) and some fuzzing-goals are automatically tested in our CI.  
You can exchange `--base-url=http://localhost:8081` to `--base-url=https://nav.tum.sexy` for the full public API, or restrict your scope using a option like `--endpoint=/api/calendar/`.

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
