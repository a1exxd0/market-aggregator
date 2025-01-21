//! Supports multiple books insertion

use super::AggregatedOrderBook;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Supports insertion from synchronous contexts.
pub struct Multibook {
    curr_id: u32,
    pub subscribed: BTreeMap<u32, Arc<AggregatedOrderBook>>,
}

impl Multibook {
    pub fn new() -> Self {
        Self {
            curr_id: 1000,
            subscribed: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, book: &Arc<AggregatedOrderBook>) -> u32 {
        let id = self.get_new_id();

        self.subscribed.insert(id, Arc::clone(book));

        id
    }

    fn get_new_id(&mut self) -> u32 {
        self.curr_id += 1;
        return self.curr_id;
    }
}
