# Dependencys

Our project has a few system-level-dependencies, which are generally usefully and a few which are only used for some
parts of the project.
If you get stuck or have any questions, feel free to contact us. We are happy to help.

## General

### OS

We recommend using a Linux based OS, as we have not tested the project on Windows or Mac.
("There be dragons", but we will try to improve this part if you show us where we fail)
If you are using Windows, use [WSL](https://docs.microsoft.com/en-us/windows/wsl/install-win10) to run Linux on Windows.

Please make sure that your OS is up-to-date, before we start. (Trust me, this has fucked over multiple people...)
On Ubuntu this is as easy as running `sudo apt update && sudo apt upgrade`.

### Git

You probably already have it, but if not, install it using your package manager.

### Docker

We deploy our project using docker containers.
This means, that if you have docker installed, you can:

- Run a part of the project like the `server`, our `tileserver` or the search engine `meilisearch` locally
- Test deployment-linked changes locally

To get started with docker, you can follow the [official tutorial](https://docs.docker.com/get-started/).

## Specific (most of these are only needed for development of said part)

### Data Processing

#### Python3

The data processing scripts are written in python, and they implicitly depend on a recent version of python (~3.10).
If you don't meet this requirement, head over to the [python website](https://www.python.org/downloads/) and download
the latest version.

### Server

#### Python3

The server does have some scripts, which are written in python, and they implicitly depend on a recent version of
python (~3.10).
If you don't meet this requirement, head over to the [python website](https://www.python.org/downloads/) and download
the latest version.
We also assume that `python --version` outputs something like `Python 3.1X.Y`.

#### Rust

Our server is written in [Rust](https://youtu.be/Q3AhzHq8ogs).
To get started with Rust, you can follow the [official tutorial](https://www.rust-lang.org/learn/get-started).
To install Rust, you can use [rustup](https://rustup.rs/).

### Feedback

#### Rust

Our server is written in [Rust](https://youtu.be/Q3AhzHq8ogs).
To get started with Rust, you can follow the [official tutorial](https://www.rust-lang.org/learn/get-started).
To install Rust, you can use [rustup](https://rustup.rs/).

#### OpenSSL

The server uses OpenSSL to verify TLS certificates.

```bash
sudo apt-get install build-essential pkg-config openssl libssl-dev
```

#### SQLite

The server uses SQLite

```bash
sudo apt-get install libsqlite3-dev
```

### Webclient

#### NodeJS

We use NodeJS for the webclient.
Setting NodeJS up is a bit more complicated than setting up python/rust, but it is still pretty easy.

- On linux, you can get it through your favorite package manager.
  You normally should need to install `nodejs` and `npm`.
- On WSL, use [this guide](https://learn.microsoft.com/en-us/windows/dev-environment/javascript/nodejs-on-wsl)

#### Gulp

We (current) use Gulp to build the webclient. gulp is a task runner, which is used to automate tasks.
Gulp needs to be installed globally, so that it can be used from the command line.

Installing _Gulp_ with npm:

```bash
sudo npm install -g gulp
```
