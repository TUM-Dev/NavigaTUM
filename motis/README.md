# API client for motis v2

This API-client was automatically generated to support the motis v2 api.
It was generated using 

```bash
wget https://raw.githubusercontent.com/motis-project/motis/refs/heads/master/openapi.yaml
sed -i 's/openapi: 3.1.0/openapi: 3.0.0/' openapi.yaml
cargo progenitor -i openapi.yaml -o motis-openapi2 -n motis-openapi2 -v 2.0.0
rm openapi.yaml
```

To use it, you can use the Client
```
# tokio_test::block_on(async {
  let res = motis_openapi_progenitor::Client::new("https://api.transitous.org").plan()
  .from_place("de-DELFI_000010073203")
  .to_place("0,0,2")
  .detailed_transfers(false)
  .send().await.unwrap();
println!("{res:?}");
# });
```
