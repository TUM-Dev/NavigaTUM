# Webclient

This folder contains the JavaScript based webclient for NavigaTUM.

## Getting started
### Recommended IDE Setup

[VSCode](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (and disable Vetur) + [TypeScript Vue Plugin (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.vscode-typescript-vue-plugin).

### Type Support for `.vue` Imports in TS

TypeScript cannot handle type information for `.vue` imports by default, so we replace the `tsc` CLI with `vue-tsc` for type checking. In editors, we need [TypeScript Vue Plugin (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.vscode-typescript-vue-plugin) to make the TypeScript language service aware of `.vue` types.

If the standalone TypeScript plugin doesn't feel fast enough to you, Volar has also implemented a [Take Over Mode](https://github.com/johnsoncodehk/volar/discussions/471#discussioncomment-1361669) that is more performant. You can enable it by the following steps:

1. Disable the built-in TypeScript Extension
    1) Run `Extensions: Show Built-in Extensions` from VSCode's command palette
    2) Find `TypeScript and JavaScript Language Features`, right click and select `Disable (Workspace)`
2. Reload the VSCode window by running `Developer: Reload Window` from the command palette.

## Customize configuration

See [Vite Configuration Reference](https://vitejs.dev/config/).

## Project Setup

```sh
npm install
```

### Compile and Hot-Reload for Development

Ensure that _NavigaTUM-server_ is running in the background.
By default the webclient will connect to the server on `http://localhost:8080`.
If you want to connect to the public API instead, change `api_prefix` in `config-local.js` to `https://nav.tum.sexy/api/` and rebuild.

```sh
npm run dev
```

### Type-Check, Compile and Minify for Production

```sh
npm run build
```

### Lint with [ESLint](https://eslint.org/)

```sh
npm run lint

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

The NavigaTUM webclient is made as a single-page application based on [Vue.js](https://vuejs.org/) and [Vue Router](https://router.vuejs.org/). The CSS framework is [Spectre.css](https://picturepan2.github.io/spectre/). It is made up of a core codebase, _views_ and _modules_:

- The core codebase provides the routing functionality, as well as helper functions (e.g. to retrieve data). All of this is bundles in the `navigatum` object in JS.
- _Views_ (taking over the terminology from vue-router) are the pages displayed in NavigaTUM.
- _Modules_ provide extra functionality that is not critical or used by multiple views (e.g. the interactive map).

### Directory structure

```bash
webclient
├── build/    # 🠔 Build files will be written here
├── src/
│   ├── assets/  # 🠔 Static assets such as icons
│   ├── md/      # 🠔 Static pages written in markdown. Served at `/about/<filename>`.
│   ├── modules/
│   │   ├── autocomplete.js     # 🠔 Autocompletion for search
│   │   └── interactive-map.js  # 🠔 Interactive map based on Mapbox
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
