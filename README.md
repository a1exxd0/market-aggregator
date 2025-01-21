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
```
Binance key is not required as we use a public API.
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
- [X] Implement proper connection management with auto-reconnection and error handling
  - If "auto-reconnection" means making sure you respond to heartbeats/pings!
  - Error handling and logging definitely there.
- [X] Implement basic monitoring capabilities for connection status
  - If something fails it is logged. If we refresh it is logged.
- [X] Handle Deribit WebSocket
- [X] Handle Binance WebSocket
  - Somewhat lazy to not implement the streams version of this :( sorry
- [ ] Process real-time updates while maintaining an accurate order-book state
- [X] Let it easily accomodate more exchanges later on
- [X] Support for additional trading pairs beyond BTC-USDT
## Storage Solution
- [X] Temporary storage optimised for time-series and tick-level data
  - Sometimes, [simplicity](https://quant.stackexchange.com/questions/613/what-is-the-best-data-structure-implementation-for-representing-a-time-series) is the key to producing a product best-suited to purpose! I'll be using arrays.
- [X] Fast retrieval strategy for stored data through partitioning/indexing etc
  - This is something I can integrate behind an interface wrapper for the above. The time complexity of these kind of partition-indexing operations are O(log(n)), and even though this isn't the be-all-end-all, sources from the stack-exchange mentioned above indicate that the cache-friendliness of a binary search against something like a B-tree/LSM-tree *might* be better for something in-memory. For something you'd want to push fast, it's also easy to implement!
## Aggregation logic
- [X] Combine order book data from exchanges into consolidated, unified view
- [X] Implement proper error handling for malformed or unexpected data
- [X] Develop an aggregation framework that supports more custom analytics or augmentation
  - I mean I haven't properly, but this isn't hard to add.
- [X] Add an additional statistic or sophisticated analytics feature like volume/imbalance between bid/ask
  - Imbalance implemented in form of bid/ask ratio
## Testing
- [X] Provide a plan outlining approach to ensure system reliability
  - Thorough unit testing (written by someone who is not me)
  - Anything failable should return a result type and be handled and logged appropriately
  - No use of Rust `unsafe`!
  - Detailed logging in general
  - Probably should tidy up the API and make a library out of it in a nice manner
  - (this is on top of stuff I've done):
    - user testing
- [X] Include basic test cases demonstrating core functionality
  - `src/lib.rs` in `mod test`, theres some stuff but if you want it printed remove the comments for the last panic call. You can also run `cargo run --features test-apis` which allow you to connect to the test APIs for the exchanges. This was a must for my testing since my binance live API didn't want to work. 
- Include examples of different types of tests
  - [X] Unit -> src/time_series_array/mod.rs
  - [X] Integration -> src/book_management/mod.rs & test/integration.rs
  - [X] Functional
    - `cargo run --features test-apis`. This is unrigorous but nevermind! I think this would be better executed by checking a constantly refreshing book state, but this requires exposing bits of types that I'd only expose for the sake of testing.
  - [X] End-to-end
    - Above kind of covers but I guess its more run the app on non-test apis and see how it looks and refreshes
  - [X] Performance
    - Example and kind of useless benchmark in benches/ts_array.rs
- [X] Include test coverage reporting
  - Run `cargo llvm-cov`
## Environment
- [X] Ensure the development environment is compatible across architectures
- [X] Design the system to be scalable for handling varying levels of market activity
  - better done with some kind of map-reduce arch i reckon? there's alternatives but imo haven't really done this justice
- [X] Implement basic CI/CD workflow
- [X] Configure build automation
- [X] Include proper logging and monitoring
