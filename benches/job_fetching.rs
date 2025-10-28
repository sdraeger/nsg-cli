use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nsg_cli::client::NsgClient;
use nsg_cli::config::Credentials;

// Mock data for testing - you'll need to adjust this based on your setup
const SAMPLE_JOB_COUNT: usize = 20;

fn benchmark_job_fetching(c: &mut Criterion) {
    // Note: This benchmark requires valid NSG credentials
    // You can skip this benchmark if credentials are not available

    let credentials_result = Credentials::load();
    if credentials_result.is_err() {
        eprintln!("Skipping benchmark: No credentials found");
        return;
    }

    let credentials = credentials_result.unwrap();
    let client_result = NsgClient::new(credentials);

    if client_result.is_err() {
        eprintln!("Skipping benchmark: Failed to create client");
        return;
    }

    let client = client_result.unwrap();

    // Test connection first
    if client.test_connection().is_err() {
        eprintln!("Skipping benchmark: Cannot connect to NSG API");
        return;
    }

    let mut group = c.benchmark_group("job_list_fetching");
    group.sample_size(10); // Reduce sample size since API calls are slow

    group.bench_function("fetch_all_jobs", |b| {
        b.iter(|| {
            let jobs = client.list_jobs();
            black_box(jobs)
        })
    });

    group.finish();
}

fn benchmark_parallel_overhead(c: &mut Criterion) {
    // Simulate the overhead of parallel vs sequential processing
    // with a simple computation (no I/O)

    let items: Vec<i32> = (0..100).collect();

    let mut group = c.benchmark_group("parallel_overhead");

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("sequential", size), size, |b, &size| {
            let items = &items[..size];
            b.iter(|| {
                let result: Vec<i32> = items.iter().map(|&x| black_box(x * x)).collect();
                black_box(result)
            });
        });

        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            group.bench_with_input(BenchmarkId::new("parallel", size), size, |b, &size| {
                let items = &items[..size];
                b.iter(|| {
                    let result: Vec<i32> = items.par_iter().map(|&x| black_box(x * x)).collect();
                    black_box(result)
                });
            });
        }
    }

    group.finish();
}

criterion_group!(benches, benchmark_job_fetching, benchmark_parallel_overhead);
criterion_main!(benches);
