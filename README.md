# NavigaTUM

[![Deployment Status](https://argocd.frank.elsinga.de/api/badge?name=navigatum)](https://argocd.frank.elsinga.de/applications/navigatum)
[![Website Uptime over 30 days](https://img.shields.io/uptimerobot/ratio/m791520694-3a5056fab92b80370cbc4c1d?label=Website%20Uptime/30d)](https://stats.uptimerobot.com/xBGmxHLMYg)
[![API Uptime over 30 days](https://img.shields.io/uptimerobot/ratio/m791522180-717477e7e0d458d047022d4a?label=API%20Uptime/30d)](https://stats.uptimerobot.com/xBGmxHLMYg)
[![CDN Uptime over 30 days](https://img.shields.io/uptimerobot/ratio/m791522182-e08c84d38117fa5d1477fe1d?label=CDN%20Uptime/30d)](https://stats.uptimerobot.com/xBGmxHLMYg)

NavigaTUM is a non-official tool developed by students for students, that aims to help you get around at [TUM](https://tum.de). Feel free to contribute.

Features:

- Interactive or RoomFinder-like maps to lookup the position of rooms or buildings
- Fast and typo-tolerant search
- Support for different room code formats as well as generic names

All functionality is also available via an API.

_Note: Not all buildings in NavigaTUM are owned by TUM, but are instead being used by TUM departments._

## Screenshots

<img alt="Screenshot of the main-index of the website" src="./resources/website-screenshots/main-index_light.png#gh-light-mode-only" width="75%"/>
<img alt="Screenshot of the main-index of the website" src="./resources/website-screenshots/main-index_dark.png#gh-dark-mode-only" width="75%"/> 
<img alt="Screenshot of a building including an internal map" src="./resources/website-screenshots/building-with-internal-map_light.png#gh-light-mode-only" width="75%"/>
<img alt="Screenshot of a building including an internal map" src="./resources/website-screenshots/building-with-internal-map_dark.png#gh-dark-mode-only" width="75%"/> 
<img alt="Screenshot of the search-page" src="./resources/website-screenshots/example-search_light.png#gh-light-mode-only" width="75%"/>
<img alt="Screenshot of the search-page" src="./resources/website-screenshots/example-search_dark.png#gh-dark-mode-only" width="75%"/>

## API Documentation and native clients

We describe our API in an [OpenAPI 3.0](https://de.wikipedia.org/wiki/OpenAPI) compliant file.  
You can find it [here](./openapi.yaml).  
Using this Specification you can generate your own client to access the API in the language of your choice.
To do this head over to the [Swagger Editor](https://editor.swagger.io/?url=https://raw.githubusercontent.com/TUM-Dev/navigatum/main/openapi.yaml) or other similar [OpenAPI tools](https://openapi.tools/).

Note: The API is still under development, and we are open to Issues, Feature Requests or Pull Requests.

## Getting started

NavigaTUM consists of three parts + deployment resources.

- `data/` contains the code to obtain and process the data
- `server/` contains the API server written in Rust, including MeiliSearch as a search backend
- `webclient/` contains a JS based web-frontend for the API
- `deployment/` contains deployment related configuration

Depending on what you want to work on, you do not need to set up all of them.
For an overview how the components work, have a look at the
[deployment documentation](deployment/README.md).

The following steps assume you have just cloned the repository and are in the
root directory of it.

### Data

In case you do not want to work on the data processing, you can instead
download the latest compiled files:

```bash
wget -P data/output https://nav.tum.sexy/cdn/api_data.json
wget -P data/output https://nav.tum.sexy/cdn/search_data.json
wget -P data/output https://nav.tum.sexy/cdn/search_synonyms.json
```

Else you can follow the steps in the [data documentation](data/).

### Server

Follow the steps in the [server documentation](server/).

### Webclient

Follow the steps in the [webclient documentation](webclient/).
If you want to only run the webclient locally, you can skip the "Data" and
"Server" steps above and edit the webclient configuration to use the public
API as is described in the webclient documentation.

### API

We format our api via [openapi-format](https://www.npmjs.com/package/openapi-format).

```bash
npm install openapi-format
openapi-format ./openapi.yaml --output ./openapi.yaml
```

To validate that the specification is being followed, use the [Swagger Editor](https://editor.swagger.io/?url=https://raw.githubusercontent.com/TUM-Dev/navigatum/main/openapi.yaml) in tandem with [stoplight](stoplight.io), as they are both very imperfect tools.

To make sure that this specification is up-to-date and without holes, we run [schemathesis](https://github.com/schemathesis/schemathesis) using the following command on API Server provided by the "Server" step or the public API:

```bash
python -m venv venv
source venv/bin/activate
pip install schemathesis
st run --workers=auto --base-url=http://localhost:8080 --checks=all ../openapi.yaml
```

Some fuzzing-goals may not be available for you locally, as they require prefix-routing (f.ex.`/cdn` to the CDN).  
You can exchange `--base-url=http://localhost:8080` to `--base-url=https://nav.tum.sexy` for the full public API, or restrict your scope using a option like `--endpoint=/api/search`.

## License

All code is licensed under the GNU GPL v3:

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
