use criterion::{Criterion, criterion_group, criterion_main};
use market_aggregator::time_series_array::TimeSeriesArray;
use std::time::Duration;

fn bench_inserts(c: &mut Criterion) {
    c.bench_function("inserts", |b| {
        let mut arr = TimeSeriesArray::new();

        for i in 0..100 {
            let _ = arr.insert(Duration::from_millis(i), &10.0);
        }
    });
}

criterion_group!(benches, bench_inserts);
criterion_main!(benches);
