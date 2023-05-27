# Webclient

This folder contains the JavaScript based webclient for NavigaTUM.

## Getting started

### Prerequisites

For getting started, there are some system dependencys which you will need.
Please follow the [system dependencys docs](/resources/documentation/Dependencys.md) before trying to run this part of our project.

### Recommended IDE Setup

[VSCode](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (and disable Vetur) + [TypeScript Vue Plugin (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.vscode-typescript-vue-plugin).
Most modern IDEs (like PyCharm) should work as well and have a Plugin.

## Dependencies

### Prerequisites

For getting started, there are some system dependencys which you will need.
Please follow the [system dependencys docs](/resources/documentation/Dependencys.md) before trying to run this part of our project.

### Installing Dependency's

```bash
npm install
```

## Run

Ensure that _NavigaTUM-server_ is running in the background:

- either via following the [guide to local development](../server/README.md), or
- via [docker](https://docs.docker.com/)  
   _docker isolates the network, but we want these two containers to communicate to each other without being as brittle as IPs._  
   _Naming the `navigatum-mieli-search` container `search` makes us able to connect to it via <`http://search:7700`> from the server_
  ```bash
  docker network create navigatum-net
  docker run -it --rm -p 7700:7700 --name search --network navigatum-net ghcr.io/tum-dev/navigatum-mieli-search:main
  docker run -it --rm -p 8080:8080 --network navigatum-net -e MIELI_SEARCH_ADDR=search ghcr.io/tum-dev/navigatum-server:main
  ```

By default, the webclient will connect to the server on `http://localhost:8080`.  
If you want to connect to the public API instead, change `VITE_APP_URL` in [`env/.env`](./env/.env) to `https://nav.tum.de`.

```sh
npm run dev
```

### Type-Check, Compile and Minify for Production

```sh
npm run build
```

### Linting with [ESLint](https://eslint.org/)

```sh
npm run lint
```

### Update the API's type definitions

From the folder of this README, run:

```sh
npx openapi-typescript ../openapi.yaml --output ./src/api_types/index.ts --export-type --immutable-types --support-array-length
```

## Build files & Serving release build

We create a lot of index HTML files in the build process.
Each of those files are similar but differ in some aspects.  
If you serve the release build with a webserver (such as Nginx) you need to select the correct files based on the request URL and headers.

```plain
<theme>-<lang>.html
   ↑       ↑
   │       └── The page language. Either "de" or "en" at the moment.
   │           It should be selected based on the "lang" Cookie or else the "Accept-Language" header.
   └── The page theme. Either "light" or "dark" at the moment.
       It should be selected based on the "theme" Cookie ("light" by default).
```

The language-selector is working in development and this differentialtion is only happening in the build.  
For the theme we can not do so for some reason (If you know of a better way, hit us up).  
To test a different theme, you can change `$theme` [here](./src/assets/variables.scss). Values are `light` and `dark`.

## Architecture

The NavigaTUM webclient is made as a single-page application based on [Vue.js](https://vuejs.org/) and [Vue Router](https://router.vuejs.org/).  
For state management we use [pinia](https://pinia.vuejs.org/) and our CSS framework is [Spectre.css](https://picturepan2.github.io/spectre/).

### Directory structure (only the important parts)

```plain
webclient
├── public/         # 🠔 Static assets such as icons, which cannot get inlined
├── src/
│   ├── codegen/    # 🠔 code generated via openapi.yaml for typechecking reasons
│   ├── assets/     # 🠔 Static assets such as icons
│   │   ├── md/                 # 🠔 Static pages written in markdown. Served at `/about/<filename>`.
│   │   ├── variables.scss      # 🠔 Include-script for Spectre.CSS
│   │   ├── main.scss           # 🠔 Sass CSS code for all non-view parts
│   │   ├── spectre-all.scss    # 🠔 Include-script for Spectre.CSS
│   │   └── logo.svg            # 🠔 Our Logo
│   ├── components/ # 🠔 Vue components, which are used in views.
│   ├── views/      # 🠔 The views are parts of App.vue, which are loaded dynamically based on our routes.
│   ├── router.ts   # 🠔 The views are parts of App.vue, which are loaded dynamically based on our routes.
│   ├── App.vue     # 🠔 Main view
│   └── main.ts     # 🠔 Inialization of Vue.js. This is the entrypoint of our app, from which App.vue and associated Views/Components are loaded
├── vite.config.ts  # 🠔 Build configuration
├── gulpfile.js     # 🠔 Gulp configuration
└── package.json    # 🠔 Node package definition and dependencies
```

Note that new views are automatically included in the build, but they are not routed.  
To add a new view, you need to add a new route in `src/router.ts`.

## Testing

For this Component, the tests consist mainly of hot-path e2e tests and tests of critical components.
PRs improving this part of the Project are very likely to be accepted.
The reason behind these tests is that they fundamentally increase the future productivity by allowing faster review cycles.

### Running Tests

There are a few ways of running cypress

#### Running headless

For running headless, it is assumed, that you are on a normal machine (not a mac) and have [Chrome](https://www.google.com/intl/de/chrome/) + [Firefox Developer Edition](https://www.mozilla.org/de/firefox/developer/) installed.
```bash
npm run test
```
There are also some subtargets preconfigured like `cy:run:chrome` and `cy:run:firefox`, but likely for debugging you want the second mode.

#### Running headed

The interface for interacting with cypress can be opened via
```bash
npm run cy:open
```

### Writing Tests
Our Cypress test suite is located in the cypress directory, organized into different files and folders based on the features and components being tested.
Each test file follows the naming convention <name>.spec.ts.

Cypress provides a comprehensive API for interacting with and asserting against elements on the web page.
You can find detailed documentation and examples in the official Cypress documentation: <https://docs.cypress.io>

When writing new tests, please ensure to follow our established conventions and guidelines to maintain consistency across the codebase.
Additionally, make sure to write descriptive test cases that cover different scenarios and edge cases to thoroughly validate the functionality of our frontend.


### Continuous Integration
We have integrated Cypress tests into our CI/CD pipeline to ensure that all changes to the frontend are thoroughly tested before deployment.
Every push and pull request triggers a build that runs the Cypress tests automatically.
This helps us catch any regressions or issues early in the development process.

### Reporting Issues
If you encounter any problems while running the Cypress tests or have suggestions for improving the testing framework, please open an issue/pull request on this repository.
We appreciate your feedback and contributions.
