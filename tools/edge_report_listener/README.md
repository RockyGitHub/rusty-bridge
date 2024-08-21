# Edge Reporter Receiver
This tool serves as a simple way to test the EdgeReporter in the rusty-bridge
Run this with
```sh
cargo run
# for more verbosity
RUST_LOG=debug cargo run
```

## Routes
POST to /edge_report/static
POST to /edge_report/dynamic

GET from /
This will return all edges and their data


