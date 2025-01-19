[![Test](https://github.com/a1exxd0/market-aggregator/actions/workflows/rust.yml/badge.svg)](https://github.com/a1exxd0/market-aggregator/actions/workflows/rust.yml)
[![Docs](https://github.com/a1exxd0/market-aggregator/actions/workflows/pages.yml/badge.svg)](https://github.com/a1exxd0/market-aggregator/actions/workflows/pages.yml)
# market-aggregator
Completed as part of a take-home assessment.


# Dependency installation
Firstly, you'll need the [Rust toolchain](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed. I'd be more specific but I don't know what OS you are using.

Then, for code coverage tooling:
```sh
cargo install cargo-llvm-cov
```

To set up APIs, you should create a dotenv file `.env` inside `market-aggregator`, and fill in your details as follows:
```sh
DERIBIT_CLIENT_ID=xxxxxx
DERIBIT_API_KEY=xxxxxx
BINANCE_API_KEY=xxxxxx
```
# Build and run
Simple as:
```rust
cargo build
cargo run
```
If you want to run tests:
```rust
cargo test
```
# A checklist
## WebSocket Integration
- [ ] Implement proper connection management with auto-reconnection and error handling
- [ ] Implement basic monitoring capabilities for connection status
- [X] Handle Deribit WebSocket
- [ ] Handle Binance WebSocket
- [ ] Process real-time updates while maintaining an accurate order-book state
- [X] Let it easily accomodate more exchanges later on
- [ ] Support for additional trading pairs beyond BTC-USDT
## Storage Solution
- [X] Temporary storage optimised for time-series and tick-level data
  - Sometimes, [simplicity](https://quant.stackexchange.com/questions/613/what-is-the-best-data-structure-implementation-for-representing-a-time-series) is the key to producing a product best-suited to purpose! I'll be using arrays.
- [X] Fast retrieval strategy for stored data through partitioning/indexing etc
  - This is something I can integrate behind an interface wrapper for the above. The time complexity of these kind of partition-indexing operations are O(log(n)), and even though this isn't the be-all-end-all, sources from the stack-exchange mentioned above indicate that the cache-friendliness of a binary search against something like a B-tree/LSM-tree *might* be better for something in-memory. For something you'd want to push fast, it's also easy to implement!
## Aggregation logic
- [ ] Combine order book data from exchanges into consolidated, unified view
- [ ] Implement proper error handling for malformed or unexpected data
- [ ] Develop an aggregation framework that supports more custom analytics or augmentation
- [ ] Add an additional statistic or sophisticated analytics feature like volume/imbalance between bid/ask
## Testing
- [ ] Provide a plan outlining approach to ensure system reliability
- [ ] Include basic test cases demonstrating core functionality
- Include examples of different types of tests
  - [X] Unit -> src/time_series_array/mod.rs
  - [ ] Integration
  - [ ] Functional
  - [ ] End-to-end
  - [ ] Performance
- [X] Include test coverage reporting
  - Run `cargo llvm-cov`
## Environment
- [X] Ensure the development environment is compatible across architectures
- [ ] Design the system to be scalable for handling varying levels of market activity
- [X] Implement basic CI/CD workflow
- [X] Configure build automation
- [X] Include proper logging and monitoring
