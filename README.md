# NavigaTUM

[![Website Uptime over 30 days](https://nav-monitoring.mm.rbg.tum.de/api/badge/5/uptime/720?label=Website%20Uptime/30&labelSuffix=d)](https://nav-monitoring.mm.rbg.tum.de/status/navigatum)
[![API Uptime over 30 days](https://nav-monitoring.mm.rbg.tum.de/api/badge/2/uptime/720?label=API%20Uptime/30&labelSuffix=d)](https://nav-monitoring.mm.rbg.tum.de/status/navigatum)
[![CDN Uptime over 30 days](https://nav-monitoring.mm.rbg.tum.de/api/badge/1/uptime/720?label=CDN%20Uptime/30&labelSuffix=d)](https://nav-monitoring.mm.rbg.tum.de/status/navigatum)

NavigaTUM is a tool developed by students for students, to help you get around at [TUM](https://tum.de).
Feel free to contribute, we are open to new people 😄.

> [!NOTE]
> Not all buildings in NavigaTUM are owned by TUM, but are instead being used by TUM departments.

## Features/Roadmap

- [x] 🗺️ Interactive/static maps to look up the position of rooms or buildings
- [x] 🔍 Fast and typo-tolerant search
- [x] 💾 Support for different room code formats as well as generic names
- [x] 🤖 All functionality is also available via an open and well documented API
- [x] 🗘 Automatically update the data from upstream datasources
- [ ] 🗨️ Allow students/staff to easily submit feedback and data patches
- [ ] 🏫 Generate maps from CAD data sources
- [ ] 🚶🏻 Generate turn by turn navigation advice for navigating end to end

If you'd like to help out or join us in this adventure, we would love to talk to you.

## Screenshots

<img alt="Screenshot of the main-index of the website" src="./resources/website-screenshots/main-index_light.png#gh-light-mode-only" width="50%"/><img alt="Screenshot of the main-index of the website" src="./resources/website-screenshots/main-index_dark.png#gh-dark-mode-only" width="50%"/><img alt="Screenshot of a building including an internal map" src="./resources/website-screenshots/building-with-internal-map_light.png#gh-light-mode-only" width="50%"/><img alt="Screenshot of a building including an internal map" src="./resources/website-screenshots/building-with-internal-map_dark.png#gh-dark-mode-only" width="50%"/>
<img alt="Screenshot of the search-page" src="./resources/website-screenshots/example-search_light.png#gh-light-mode-only" width="100%"/><img alt="Screenshot of the search-page" src="./resources/website-screenshots/example-search_dark.png#gh-dark-mode-only" width="100%"/>

## API Documentation and native clients

You can consume our API Documentation in two ways:

- Head over to [our Website](https://nav.tum.de/api) and look at the interactive documentation
- We also describe our API in an [OpenAPI 3.0](https://de.wikipedia.org/wiki/OpenAPI) compliant file.  
  You can find it [here](openapi.yaml).  
  Using this Specification you can generate your own client to access the API in the language of your choice.
  To do this head over to
  the [Swagger Editor](https://editor.swagger.io/?url=https://raw.githubusercontent.com/TUM-Dev/navigatum/main/openapi.yaml)
  or other similar [OpenAPI tools](https://openapi.tools/).

Note: The API is still under development, and we are open to Issues, Feature Requests or Pull Requests.

## Getting started

### Overview

NavigaTUM consists of three main parts + deployment resources.

Depending on what you want to work on, you **do not need to set up all of them**.

- `data/` contains the code to obtain and process the data
- `server/` contains the APIs written in Rust
- `webclient/` contains a JS based web-frontend for the API
- `deployment/` contains deployment related configuration
- `map/` contains information about our own map, how to style it and how to run it

Let's go through them one by one, but first, you need to clone the repository:

```bash
git clone https://github.com/TUM-Dev/Navigatum.git
cd Navigatum
```

### Data Processing

In case you do not want to work on the data processing, you can instead
download the latest compiled files:

```bash
wget -P data/output https://nav.tum.de/cdn/api_data.json
wget -P data/output https://nav.tum.de/cdn/search_data.json
```

Else you can follow the steps in the [data documentation](data/README.md).

### Server

If you want to work on the webclient only (and not server or data), you don't need to set up the server.
You can instead either use the public API (see the [webclient documentation](webclient/README.md#Testing)) or use our
ready-made docker images to run the server locally:

> [!NOTE]
> Because we have a tileserver, running the following command is required once.
>
> ```bash
> docker run -it --rm -v tileserver-src:/data ubuntu:latest sh -c "apt -qq update && apt -qq install wget -y && wget --progress=bar:force:noscroll --timestamping --directory-prefix=/data --compression=auto --continue --tries=5  https://nav.tum.de/maps/vol/output.mbtiles"
> ```

```bash
docker compose -f docker-compose.local.yml up --build
```

Else you can follow the steps in the [server documentation](server/README.md).

### Webclient

Follow the steps in the [webclient documentation](webclient/README.md).
If you want to only run the webclient locally, you can skip the "Data" and "Server" steps above and use docker (as seen
above) or you can [edit the webclient configuration](webclient/README.md#testing) to point to production.

### Formatting

We have multiple programming languages in this repository, and we use different tools to format them.

since we use [pre-commit](https://pre-commit.com/) to format our code, you can install it in an virtual environment
with:

```bash
python3 -m venv venv
source venv/bin/activate
pip install -r data/requirements.txt -r server/test/requirements.txt -r requirements-dev.txt # for mypy the server and data requirements are needed
```

To format all files, run the following command:

```bash
pre-commit run --all-files
```

You can also automatically **format files on every commit** by running the following command:

```bash
pre-commit install
```

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
