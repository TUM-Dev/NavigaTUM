# cad2osm

`cad2osm` is a Rust-based tool to convert CAD files from the IFC format to OSM format.

## Installation

To install `cad2osm`, ensure you have [Rust](https://www.rust-lang.org/) installed. Then, run:

```sh
cargo install --git https://github.com/yourusername/cad2osm
```

## Usage
After installation, you can use the cad2osm command:

```shell
cad2osm [OPTIONS] <FLOOR_LEVEL> <PATH>
```
### Arguments:

- `<FLOOR_LEVEL>`  floor level to assign to most of the structure
- `<PATH>`         path to the file to read

### Options:
- `--floor-name <NAME>` name of the `FLOOR_LEVEL`
- `-v`, `--verbose`     Increase the verbosity. Can be Repeated multiple times
- `-h`, `--help`        Print help

## Development

> [!NOTE]
> Contributions are welcome!
> Please open an issue or submit a pull request to discuss what you need/want changed how.

To start contributing, clone the repository:

```shell
git clone https://github.com/TUM-Dev/navigatum.git
cd NavigaTUM/cad2osm
```

Build and test the project:

```shell
cargo build
cargo test
```
To run the program locally:

```shell
cargo run -- 3 ./example.ifc --floor-name "Third Floor"
```
