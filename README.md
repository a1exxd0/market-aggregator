[![Test](https://github.com/a1exxd0/market-aggregator/actions/workflows/rust.yml/badge.svg)](https://github.com/a1exxd0/market-aggregator/actions/workflows/rust.yml)
[![Docs](https://github.com/a1exxd0/market-aggregator/actions/workflows/pages.yml/badge.svg)](https://github.com/a1exxd0/market-aggregator/actions/workflows/pages.yml)
# market-aggregator
Completed as part of a take-home assessment.


# setup
Firstly, you'll need the [Rust toolchain](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed. 

Thats it :) 

# A checklist
## WebSocket Integration
- [ ] Implement proper connection management with auto-reconnection and error handling
- [ ] Implement basic monitoring capabilities for connection status
- [ ] Handle Deribit WebSocket
- [ ] Handle Binance WebSocket
- [ ] Process real-time updates while maintaining an accurate order-book state
- [ ] Let it easily accomodate more exchanges later on
- [ ] Support for additional trading pairs beyond BTC-USDT
## Storage Solution
- [ ] Temporary storage optimised for time-series and tick-level data
- [ ] Fast retrieval strategy for stored data through partitioning/indexing etc
## Aggregation logic
- [ ] Combine order book data from exchanges into consolidated, unified view
- [ ] Implement proper error handling for malformed or unexpected data
- [ ] Develop an aggregation framework that supports more custom analytics or augmentation
- [ ] Add an additional statistic or sophisticated analytics feature like volume/imbalance between bid/ask
## Testing
- [ ] Provide a plan outlining approach to ensure system reliability
- [ ] Include basic test cases demonstrating core functionality
- [ ] Include examples of different types of tests (unit, integration, functional, end-to-end, performance)
- [ ] Include test coverage reporting
## Environment
- [ ] Ensure the development environment is compatible across architectures
- [ ] Design the system to be scalable for handling varying levels of market activity
- [X] Implement basic CI/CD workflow
- [X] Configure build automation
- [ ] Include proper logging and monitoring