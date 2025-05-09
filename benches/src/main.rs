use anyhow::Result;
use std::{fs::File, io::Write, time::Duration};

mod proof_gen_time;
mod root_calc_time;
mod update_time;

// Account counts to benchmark
const ACCOUNT_COUNTS: &[usize] = &[10_000, 50_000, 100_000, 200_000, 500_000];
// Number of iterations per benchmark
const ITERATIONS: usize = 5;

fn main() -> Result<()> {
    println!("Starting MPT database implementation performance comparison");
    println!("=========================================================");

    // Create results directory
    std::fs::create_dir_all("benchmark_results")?;

    // Run update time benchmarks
    println!("\nRunning MPT update time benchmarks");
    println!("--------------------------------");

    let mdbx_update_results = update_time::benchmark_mdbx_update_time(ACCOUNT_COUNTS, ITERATIONS)?;
    let rocksdb_update_results =
        update_time::benchmark_rocksdb_update_time(ACCOUNT_COUNTS, ITERATIONS)?;

    // Write update time results to CSV
    write_results_to_csv(
        "benchmark_results/update_time.csv",
        &mdbx_update_results,
        &rocksdb_update_results,
        "Update Time (s)",
    )?;

    // Run root calculation time benchmarks
    println!("\nRunning MPT root calculation time benchmarks");
    println!("-----------------------------------------");

    let mdbx_root_calc_results =
        root_calc_time::benchmark_mdbx_root_calc_time(ACCOUNT_COUNTS, ITERATIONS)?;
    let rocksdb_root_calc_results =
        root_calc_time::benchmark_rocksdb_root_calc_time(ACCOUNT_COUNTS, ITERATIONS)?;

    // Write root calculation time results to CSV
    write_results_to_csv(
        "benchmark_results/root_calc_time.csv",
        &mdbx_root_calc_results,
        &rocksdb_root_calc_results,
        "Root Calculation Time (s)",
    )?;

    // Run proof generation time benchmarks
    println!("\nRunning MPT proof generation time benchmarks");
    println!("-----------------------------------------");

    let mdbx_proof_gen_results =
        proof_gen_time::benchmark_mdbx_proof_gen_time(ACCOUNT_COUNTS, ITERATIONS)?;
    let rocksdb_proof_gen_results =
        proof_gen_time::benchmark_rocksdb_proof_gen_time(ACCOUNT_COUNTS, ITERATIONS)?;

    // Write proof generation time results to CSV
    write_results_to_csv(
        "benchmark_results/proof_gen_time.csv",
        &mdbx_proof_gen_results,
        &rocksdb_proof_gen_results,
        "Proof Generation Time (s)",
    )?;

    println!("\nBenchmark completed! Results written to 'benchmark_results' directory.");
    println!("Please check the CSV files for detailed results.");

    Ok(())
}

/// Write benchmark results to a CSV file
fn write_results_to_csv(
    filename: &str,
    mdbx_results: &[(usize, Duration)],
    rocksdb_results: &[(usize, Duration)],
    metric_name: &str,
) -> Result<()> {
    // Create full path and ensure parent directories exist
    let full_path = std::path::Path::new(filename);
    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Create or truncate the file
    let mut file = File::create(full_path)?;

    writeln!(file, "Account Count,MDBX {} (s),RocksDB {} (s),Speedup", metric_name, metric_name)?;

    for i in 0..mdbx_results.len() {
        let (account_count, mdbx_time) = mdbx_results[i];

        let rocksdb_time = rocksdb_results
            .iter()
            .find(|(count, _)| *count == account_count)
            .map(|(_, time)| *time)
            .unwrap_or(Duration::from_secs(0));

        // Convert durations to milliseconds
        let mdbx_ms = mdbx_time.as_secs_f64() * 1000.0;
        let rocksdb_ms = rocksdb_time.as_secs_f64() * 1000.0;

        let speedup = if rocksdb_ms > 0.0 && mdbx_ms > 0.0 { mdbx_ms / rocksdb_ms } else { 0.0 };

        writeln!(file, "{},{:.3},{:.6},{:.2}x", account_count, mdbx_ms, rocksdb_ms, speedup)?;
    }

    println!("Results written to {}", filename);
    Ok(())
}
