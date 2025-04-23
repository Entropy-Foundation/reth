# Benchmark

## Update time

```sh
cargo b -p mpt-db-comparison --jobs 8
cargo r
```

<details>
<summary>
Logs
</summary>

```
   Compiling mpt-db-comparison v0.1.0 (/home/psychopunk_sage/dev/Workplace/Supra/reth/benches)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.84s
     Running `/home/psychopunk_sage/dev/Workplace/Supra/reth/target/debug/mpt-db-comparison`
Starting MPT database implementation performance comparison
=========================================================

Running MPT update time benchmarks
--------------------------------
Benchmarking MDBX update time with 10000 accounts
  Iteration 1/5
    Completed in 0.01s
  Iteration 2/5
    Completed in 0.01s
  Iteration 3/5
    Completed in 0.01s
  Iteration 4/5
    Completed in 0.01s
  Iteration 5/5
    Completed in 0.01s
Average update time for 10000 accounts: 0.01s
Benchmarking MDBX update time with 50000 accounts
  Iteration 1/5
    Completed in 0.07s
  Iteration 2/5
    Completed in 0.07s
  Iteration 3/5
    Completed in 0.08s
  Iteration 4/5
    Completed in 0.08s
  Iteration 5/5
    Completed in 0.08s
Average update time for 50000 accounts: 0.07s
Benchmarking MDBX update time with 100000 accounts
  Iteration 1/5
    Completed in 0.15s
  Iteration 2/5
    Completed in 0.15s
  Iteration 3/5
    Completed in 0.15s
  Iteration 4/5
    Completed in 0.15s
  Iteration 5/5
    Completed in 0.15s
Average update time for 100000 accounts: 0.15s
Benchmarking MDBX update time with 200000 accounts
  Iteration 1/5
    Completed in 0.30s
  Iteration 2/5
    Completed in 0.31s
  Iteration 3/5
    Completed in 0.30s
  Iteration 4/5
    Completed in 0.31s
  Iteration 5/5
    Completed in 0.31s
Average update time for 200000 accounts: 0.31s
Benchmarking MDBX update time with 500000 accounts
  Iteration 1/5
    Completed in 0.78s
  Iteration 2/5
    Completed in 0.78s
  Iteration 3/5
    Completed in 0.78s
  Iteration 4/5
    Completed in 0.76s
  Iteration 5/5
    Completed in 0.75s
Average update time for 500000 accounts: 0.77s
RocksDB benchmark - implementation will be similar to MDBX but with RocksDB backend
Results written to benchmark_results/update_time.csv
```

</details>