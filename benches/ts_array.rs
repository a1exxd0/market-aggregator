use std::{hint::black_box, time::Duration};
use criterion::{criterion_group, criterion_main, Criterion};
use market_aggregator::time_series_array::TimeSeriesArray;


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