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
If you want to connect to a local version instead, change the environemnt variable `NUXT_PUBLIC_{API,CDN,FEEDBACK,MAPS}_URL` to the appropriate value.

To get a local server running, please:

- either via following the [guide to local development](../server/README.md), or
- via [docker](https://docs.docker.com/) by commenting out the webclient from the docker-compose-file and running   
  ```bash
  docker compose -f docker-compose.local.yml up --build
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
pnpm run type-refresh
```

## Architecture

The NavigaTUM webclient is made as a nuxt3 server side rendered application based on [Vue.js](https://vuejs.org/) and [Vue Router](https://router.vuejs.org/).
Our CSS framework is [Tailwind](https://tailwindcss.com/).

### Directory structure (only the important parts)

```plain
webclient
├── public/        # 🠔 Static assets such as icons, which cannot get inlined
├── api_types/     # 🠔 code generated via openapi.yaml for typechecking reasons
├── content/       # 🠔 Static pages written in markdown. Served at `/about/<filename>`.
├── assets/        # 🠔 Static assets such as icons
│   ├── main.scss  # 🠔 Sass CSS code for all non-view parts
│   └── logos      # 🠔 The Logos used by the app
├── components/    # 🠔 Vue components, which are used in views.
├── pages/         # 🠔 The pages are parts of App.vue, which are loaded based their file names.
├── nuxt.config.ts # 🠔 core configuration of nuxt
└── package.json   # 🠔 Node package definition and dependencies
```

Note that new views are automatically included in the build, but they are not routed.  
To add a new view, you need to add a new route in `router.ts`.

## Testing

> [!NOTE]
> cypress is currently temporarily disabled to help in the nuxt transition

For this part of the project, the tests consist mainly of hot-path e2e tests and tests of critical components.
PRs improving the coverage are very likely to be accepted.
The reason behind these tests is that they fundamentally increase the future productivity by allowing faster review cycles.

### Continuous Integration

Every push and pull request triggers a build that runs linting issues (cypress is currently temporarily disabled to help in the nuxt transition).
This helps us catch any regressions or issues early in the development process.

### Reporting Issues

If you encounter any problems while running the Cypress tests or have suggestions for improving the testing framework, please open an issue/pull request on this repository.
We appreciate your feedback and contributions.
