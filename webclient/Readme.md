# Navigatum Webclient
This repository contains the JavaScript based webclient for Navigatum.

## Getting started

### Dependencies
You need the following dependencies to get started:
- Node (for Gulp)
- [Yarn](https://yarnpkg.com/getting-started/install)
- [Gulp](https://gulpjs.com/)
- Python (for testing)

Installing *Yarn* and *Gulp* with npm:
```bash
sudo npm install -g yarn gulp
```

### Test data
TBD

### Building

Install all  packages:
```bash
yarn
# alternatively, if you do not use yarn:
npm install
```

Copy the configuration file:
```bash
# The default configuration is pre-configured for local testing.
cp config.default.js config.js
```

And run Gulp to build the client. The build files will be written to `build/`.
```bash
# Run development build
gulp
# or run release build
gulp release
```

### Testing
If you do a development build you can use a simple webserver to test the build.

Ensure that *navigatum-server* is running in the background. By default the webclient will connect to the server on `http://localhost:8080`. If not you can change the configuration in `config.js` and re-build.

Now run:
```bash
python -m http.server
```

and open http://localhost:8000/build/index-view-main-light-de.html in your browser.

## Build files & Serving release build
Gulp creates a lot of index HTML files in the build process.
Each of those files are similar but differ in some aspects.
If you serve the release build with a webserver (such as Apache or Nginx) you need
to select the correct files based on the request URL and headers.

```plain
index-view-<view>-<theme>-<lang>.html
            ↑      ↑       ↑
            │      │       └── The page language. Either "de" or "en" at the
            │      │           moment. It should be selected based on the
            │      │           "lang" Cookie or else the "Accept-Language" header.
            │      └── The page theme. Either "light" or "dark" at the moment.
            │          It should be selected based on the "theme" Cookie and is
            │          "light" by default.
            └── The first loaded view (see architecture below). It does technically
                not matter which view is selected here, but this allows to efficiently
                preload resources and optimize the order of resources during initial
                pageload.
```

When running locally on a development build you can use the language and theme of
your choice as well as any view.

## Architecture
The Navigatum webclient is made as a single-page application based on [Vue.js](https://vuejs.org/) and [Vue Router](https://router.vuejs.org/). The CSS framework is [Spectre.css](https://picturepan2.github.io/spectre/). It is made up of a core codebase, *views* and *modules*:

- The core codebase provides the routing functionality, as well as helper functions (e.g. to retrieve data). All of this is bundles in the `navigatum` object in JS.
- *Views* (taking over the terminology from vue-router) are the pages displayed in Navigatum.
- *Modules* provide extra functionality that is not critical or used by multiple views (e.g. the interactive map).

### Directory structure
```bash
navigatum-web
├── build/    # 🠔 Build files will be written here
├── src/
│   ├── assets/  # 🠔 Static assets such as icons
│   ├── md/      # 🠔 Static pages written in markdown. Served at `/about/<filename>`.
│   ├── modules/
│   │   ├── autocomplete.js     # 🠔 Autocompletion for search
│   │   └── interactive-map.js  # 🠔 Interactive map based on Leaflet
│   ├── views/  # 🠔 See below
│   ├── core.js             # 🠔 Core JS code (and JS entrypoint)
│   ├── feedback.js         # 🠔 JS for the feedback form (separated from the rest of
│   │                       #    the code to work even when the core JS fails).
│   ├── history-states.js   # 🠔 Preseve state on back-/forward navigation
│   ├── i18n.yaml           # 🠔 Translation strings for the core code
│   ├── index.html          # 🠔 index.html template
│   ├── init-call.js        # 🠔 Special helper-script for init on page-load
│   ├── legacy.js           # 🠔 Special helper-script to automatically include some
│   │                       #    polyfills for older browsers.
│   ├── main.scss           # 🠔 Sass CSS code for all non-view parts
│   ├── spectre-all.scss    # 🠔 Include-script for Spectre.CSS
│   └── variables.scss      # 🠔 Sass CSS variable definitions (also defines themes)
├── vendor/       # 🠔 External libraries
├── config.js     # 🠔 Build configuration
├── gulpfile.js   # 🠔 Gulp configuration
└── package.json  # 🠔 Node package definition and dependencies
```

'Views' (pages) are located in `src/views` where each view has its own subdirectory called `view-<name>`:
```bash
view-example
├── i18n-example.yaml  # 🠔 Translation strings for each language
├── view-example.inc   # 🠔 The HTML Template of the view
├── view-example.js    # 🠔 The JS Sources of the view
└── view-example.scss  # 🠔 The Sass CSS Sources of the view
```

Note that new views are automatically included in the build, but new JS files
in the `src/` directory are not. If you add a new JS file there you need to include
it in `gulpfile.js`.

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
along with this program.  If not, see https://www.gnu.org/licenses/.
