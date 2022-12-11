# Server

This folder contains the feedback-API server for NavigaTUM.

This is separated from the server because:

- it has virtually no shared dependencies (natural faultline)
- this way, we can deploy the feedback-API independently of the main server (both in time, scaling and reliability)
- security: this way, we can increase our isolation

## Getting started

### Prerequisites

For getting started, there are some system dependencys which you will need.
Please follow the [system dependencys docs](resources/documentation/Dependencys.md) before trying to run this part of our project.

### Starting the server

Run `cargo run` to start the server.
The server should now be available on `localhost:6000`.

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
st run --workers=auto --base-url=http://localhost:6000 --checks=all ../openapi.yaml
```

Some fuzzing-goals may not be available for you locally, as they require prefix-routing (f.ex.`/cdn` to the CDN) and some fuzzing-goals are automatically tested in our CI.  
You can exchange `--base-url=http://localhost:6000` to `--base-url=https://nav.tum.sexy` for the full public API, or restrict your scope using a option like `--endpoint=/api/feedback/`.

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
