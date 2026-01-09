## API client for motis v3

This API-client was automatically generated to support the motis v3 api.
It was generated (mostly, some manual editing was done).

See <https://github.com/oxidecomputer/progenitor> for docs on the generator used

The following code can generate the client:

```bash
wget https://raw.githubusercontent.com/motis-project/motis/refs/heads/master/openapi.yaml
sed -i 's/openapi: 3.1.0/openapi: 3.0.0/' openapi.yaml

rm -fr src Cargo.*
cargo progenitor -i openapi.yaml -o motis-openapi-progenitor -n motis-openapi-progenitor -v 0.3.1  --interface builder
mv motis-openapi-progenitor/* .
rm -fr motis-openapi-progenitor

(
  echo '#![forbid(unsafe_code)]'
  echo '#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]'
  cat src/lib.rs
) > tmpfile
mv tmpfile src/lib.rs
rm openapi.yaml
```

To use it, you can use the `Client` as following

```rust
#[tokio::main]
async fn main() {
    let res = motis_openapi_progenitor::Client::new("https://api.transitous.org").plan()
        .from_place("de-DELFI_000010073203") // landshut süd trains station
        .to_place("48.1371079,11.5753822,0") // munich coordinate at level 0
        .detailed_transfers(false)
        .send().await.unwrap();
    println!("{res:?}");
}
```
