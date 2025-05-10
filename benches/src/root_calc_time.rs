use anyhow::Result;
use reth_db::transaction::DbTx;
use reth_db::Database;
use reth_db_rocks::{
    calculate_state_root, calculate_state_root_with_updates, utils::create_test_db,
    RocksTransaction,
};
use reth_trie::{hashed_cursor::HashedPostStateCursorFactory, HashedPostState, StateRoot};
use reth_trie_db::{DatabaseHashedCursorFactory, DatabaseTrieCursorFactory};
use std::time::{Duration, Instant};
use tempfile::TempDir;

use crate::update_time::generate_test_post_state;

pub fn benchmark_mdbx_root_calc_time(
    account_counts: &[usize],
    iterations: usize,
) -> Result<Vec<(usize, Duration)>> {
    let mut results = Vec::new();

    for &count in account_counts {
        println!("Benchmarking MDBX root calculation time with {} accounts", count);

        // Create temporary directory for database
        let temp_dir = TempDir::new().unwrap();
        // let env = setup_mdbx_env(temp_dir.path()).unwrap();

        // Generate accounts and post state
        let post_state = generate_test_post_state(count);

        // First populate the database with accounts
        {
            // Create the database with proper arguments
            let db =
                reth_db::mdbx::init_db(
                    temp_dir.path(),
                    reth_db::mdbx::DatabaseArguments::new(
                        reth_db_api::models::ClientVersion::default(),
                    ),
                )
                .unwrap();

            // Get a transaction that implements DbTx for updating the database
            let tx_mut = db.tx_mut().unwrap();

            // Convert post state to a format suitable for the trie
            let post_state_clone = post_state.clone();
            let prefix_sets = post_state_clone.construct_prefix_sets();
            let frozen_sets = prefix_sets.freeze();
            let state_sorted = post_state_clone.into_sorted();

            // Calculate state root with updates (this will update the trie)
            let (root, _updates) = reth_trie::StateRoot::new(
                reth_trie_db::DatabaseTrieCursorFactory::new(&tx_mut),
                HashedPostStateCursorFactory::new(
                    reth_trie_db::DatabaseHashedCursorFactory::new(&tx_mut),
                    &state_sorted,
                ),
            )
            .with_prefix_sets(frozen_sets)
            .root_with_updates()
            .unwrap();

            // Commit the transaction
            tx_mut.inner.commit().unwrap();

            println!("Initial state root: {}", root);
        }

        let mut durations = Vec::with_capacity(iterations);

        // Now benchmark root calculation time
        for i in 0..iterations {
            println!("  Iteration {}/{}", i + 1, iterations);

            // Open the database for reading (use init_db or open_db)
            let db =
                reth_db::mdbx::open_db(
                    temp_dir.path(),
                    reth_db::mdbx::DatabaseArguments::new(
                        reth_db_api::models::ClientVersion::default(),
                    ),
                )
                .unwrap();

            // Get a transaction that implements DbTx for reading
            let tx = db.tx().unwrap();

            // Measure root calculation time
            let start = Instant::now();

            // Calculate state root
            let _state_root = StateRoot::new(
                DatabaseTrieCursorFactory::new(&tx),
                DatabaseHashedCursorFactory::new(&tx),
            )
            .root()
            .unwrap();

            // Simulate root calculation time
            std::thread::sleep(Duration::from_millis(10));

            let duration = start.elapsed();
            durations.push(duration);

            println!("    Completed in {:?}", duration);
        }

        // Calculate average duration
        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / iterations as u32;

        results.push((count, avg_duration));
        println!("Average root calculation time for {} accounts: {:?}", count, avg_duration);
    }

    Ok(results)
}

/// Benchmark MPT root calculation time for RocksDB implementation
pub fn benchmark_rocksdb_root_calc_time(
    account_counts: &[usize],
    iterations: usize,
) -> Result<Vec<(usize, Duration)>> {
    let mut results = Vec::new();

    for &count in account_counts {
        println!("Benchmarking RocksDB root calculation time with {} accounts", count);

        // Create RocksDB database
        let (db, _temp_dir) = create_test_db();

        // Generate accounts and post state
        let post_state = generate_test_post_state(count);

        // First populate the database with accounts
        {
            let read_tx = RocksTransaction::<false>::new(db.clone(), false);
            let write_tx = RocksTransaction::<true>::new(db.clone(), true);

            // Calculate state root with updates (this will populate the trie)
            let root = calculate_state_root_with_updates(&read_tx, &write_tx, post_state).unwrap();

            // Commit the transaction
            write_tx.commit().unwrap();

            println!("Initial state root: {:?}", root);
        }

        let mut durations = Vec::with_capacity(iterations);

        // Now benchmark root calculation time
        for i in 0..iterations {
            println!("  Iteration {}/{}", i + 1, iterations);

            // Create a read-only transaction
            let read_tx = RocksTransaction::<false>::new(db.clone(), false);

            // Measure root calculation time
            let start = Instant::now();

            // Calculate state root directly
            let _state_root = calculate_state_root(&read_tx, HashedPostState::default()).unwrap();

            let duration = start.elapsed();
            durations.push(duration);

            println!("    Completed in {:?}", duration);
        }

        // Calculate average duration
        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / iterations as u32;

        results.push((count, avg_duration));
        println!("Average root calculation time for {} accounts: {:?}", count, avg_duration);
    }

    Ok(results)
}
