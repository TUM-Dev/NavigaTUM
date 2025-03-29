# Rust API client for Motis

This crate contains the types and functions for interacting with the Motis v2 API.

These APIs are implemented:
- [ ] [optimal connections](https://redocly.github.io/redoc/?url=https://raw.githubusercontent.com/motis-project/motis/refs/heads/master/openapi.yaml#tag/routing/operation/plan)

## Features and usage

We provide two clients:
- async: [`motis_client::Motis`](https://docs.rs/motis-client/latest/motis_client/struct.Motis.html) and
- sync: [`motis_client::blocking::Motis`](https://docs.rs/motis-client/latest/motis_client/blocking/struct.Motis.html) using the [tokyo runtime](https://tokio.rs/) internally to call the async version

The second one is behind the (default-enabled) `blocking` feature, so if you don't need it, you can disable it via `default-features = false`.

We also offer the (default-enabled) `gpx` feature.
This enables [reading and writing GPX (GPS Exchange Format) files](https://docs.rs/gpx/latest/gpx/) for APIs where we have the needed context.

## Example

```rust
// an async version is available at motis_client::Motis
use motis_client::blocking::Motis;
use motis_client::route::{OptimalConnectionOptions, StopID};
use motis_client::costing::{Costing};

let motis = Motis::default();

let options = OptimalConnectionOptions::builder()
    .locations([amsterdam, utrecht])
    .costing(Costing::Motorcycle(Default::default()));

let response = motis.route(options).unwrap();
println!("{:#?}", response);
```

For further examples, please see the different clients:
- async: [`motis_client::Motis`](https://docs.rs/motis-client/latest/motis_client/struct.Motis.html) and
- sync: [`motis_client::blocking::Motis`](https://docs.rs/motis-client/latest/motis_client/blocking/struct.Motis.html)
