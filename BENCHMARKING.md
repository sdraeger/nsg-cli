# Benchmarking and Feature Flags

## Feature Flags

This project supports a compile-time feature flag to toggle between parallel and sequential job fetching:

### `parallel` (default)

Uses Rayon's `par_iter()` for parallel processing when fetching job details. This can significantly speed up operations when you have many jobs.

```bash
# Build with parallel support (default)
cargo build --release

# Explicitly enable parallel
cargo build --release --features parallel
```

### Sequential Processing

Disable the `parallel` feature for sequential processing:

```bash
# Build without parallel support
cargo build --release --no-default-features
```

## Running Benchmarks

### Prerequisites

- Valid NSG credentials configured (`nsg login`)
- At least a few jobs in your account for realistic benchmarks

### Run All Benchmarks

```bash
cargo bench
```

### Run Specific Benchmarks

```bash
# Benchmark actual job fetching (requires credentials)
cargo bench job_list_fetching

# Benchmark parallel overhead (no credentials needed)
cargo bench parallel_overhead
```

### Compare Parallel vs Sequential

```bash
# Benchmark with parallel support (default)
cargo bench --features parallel

# Benchmark without parallel support
cargo bench --no-default-features
```

The results will be saved in `target/criterion/` with HTML reports.

## Benchmark Results Interpretation

### When to Use Parallel

Parallel processing is beneficial when:
- You have many jobs (typically 10+)
- Network latency is significant
- You have multiple CPU cores available

### When to Use Sequential

Sequential processing may be better when:
- You have very few jobs (1-5)
- API rate limiting is a concern
- You want to minimize CPU usage

## Example Comparison

To compare the two approaches:

```bash
# Test with parallel (default)
time cargo run --release -- list --all

# Test without parallel
time cargo run --release --no-default-features -- list --all
```

## Viewing Benchmark Reports

After running benchmarks, open the HTML report:

```bash
open target/criterion/report/index.html
```

Or on Linux:

```bash
xdg-open target/criterion/report/index.html
```
