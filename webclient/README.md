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
pnpm install
```

## Run

Ensure that _NavigaTUM-server_ is running in the background.
By default, the webclient will connect to the server on `https://nav.tum.de`.  
If you want to connect to a local version instead, change `VITE_APP_URL` in [`env/.env`](./env/.env) to `http://localhost:3003`.

To get a local server running, please:

- either via following the [guide to local development](../server/README.md), or
- via [docker](https://docs.docker.com/)  
   _docker isolates the network, but we want these two containers to communicate to each other without being as brittle as IPs._  
   _Naming the `navigatum-mieli-search` container `search` makes us able to connect to it via <`http://search:7700`> from the server_

  ```bash
  docker network create navigatum-net
  docker run -it --rm -p 7700:7700 --name search --network navigatum-net ghcr.io/tum-dev/navigatum-mieli-search:main
  docker run -it --rm -p 8080:8080 --network navigatum-net -e MIELI_SEARCH_ADDR=search ghcr.io/tum-dev/navigatum-server:main /bin/navigatum-main-api
  ```

```sh
pnpm run dev
```

### Type-Check, Compile and Minify for Production

```sh
pnpm run build
```

### Linting with [ESLint](https://eslint.org/) and formatting via prettier

```sh
pnpm run lint
pnpm run format
```

### Update the API's type definitions

From the folder of this README, run:

```sh
npx openapi-typescript ../openapi.yaml --output ./api_types/index.ts --export-type --immutable-types --support-array-length
pnpm run lint
pnpm run format
```

## Build files & Serving release build

We create a lot of index HTML files in the build process.
Each of those files are similar but differ in some aspects.  
If you serve the release build with a webserver (such as Nginx) you need to select the correct files based on the request URL and headers.

```plain
<theme>.html
   ↑
   └── The page theme. Either "light" or "dark" at the moment.
       It should be selected based on the "theme" Cookie ("light" by default).
```

The language-selector is working in development and this differentialtion is only happening in the build.  
For the theme we can not do so for some reason (If you know of a better way, hit us up).  
To test a different theme, you can change `theme='...'` [here](./index.html). Values are `light` and `dark`.

## Architecture

The NavigaTUM webclient is made as a single-page application based on [Vue.js](https://vuejs.org/) and [Vue Router](https://router.vuejs.org/).  
For state management we use [pinia](https://pinia.vuejs.org/).
Our CSS framework is currently being migrated from [Spectre.css](https://picturepan2.github.io/spectre/) to [Tailwind](https://tailwindcss.com/). (if you're interested in helping out, please contact us ^^)

### Directory structure (only the important parts)

```plain
webclient
├── public/         # 🠔 Static assets such as icons, which cannot get inlined
├── api_types/  # 🠔 code generated via openapi.yaml for typechecking reasons
├── assets/     # 🠔 Static assets such as icons
│   ├── md/                 # 🠔 Static pages written in markdown. Served at `/about/<filename>`.
│   ├── main.scss           # 🠔 Sass CSS code for all non-view parts
│   └── logos               # 🠔 The Logos used by the app
├── components/ # 🠔 Vue components, which are used in views.
├── pages/      # 🠔 The views are parts of App.vue, which are loaded dynamically based on our routes.
├── router.ts   # 🠔 The routes of our app. This is where the views are loaded.
├── App.vue     # 🠔 Main view
├── main.ts     # 🠔 Inialization of Vue.js. This is the entrypoint of our app, from which App.vue and associated views/components are loaded
├── vite.config.ts  # 🠔 Build configuration
└── package.json    # 🠔 Node package definition and dependencies
```

Note that new views are automatically included in the build, but they are not routed.  
To add a new view, you need to add a new route in `router.ts`.

## Testing

For this part of the project, the tests consist mainly of hot-path e2e tests and tests of critical components.
PRs improving the coverage are very likely to be accepted.
The reason behind these tests is that they fundamentally increase the future productivity by allowing faster review cycles.

### Continuous Integration

Every push and pull request triggers a build that runs linting issues (cypress is currently temporarily disabled to help in the nuxt transition).
This helps us catch any regressions or issues early in the development process.

### Reporting Issues

If you encounter any problems while running the Cypress tests or have suggestions for improving the testing framework, please open an issue/pull request on this repository.
We appreciate your feedback and contributions.
