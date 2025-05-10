use crate::update_time::generate_test_post_state;
use alloy_primitives::Address;
use anyhow::Result;
use reth_db::transaction::DbTx;
use reth_db::{mdbx, Database};
use reth_db_rocks::{calculate_state_root_with_updates, utils::create_test_db, RocksTransaction};
use reth_trie::proof::Proof;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Benchmark MPT proof generation time for MDBX implementation
pub fn benchmark_mdbx_proof_gen_time(
    account_counts: &[usize],
    iterations: usize,
) -> Result<Vec<(usize, Duration)>> {
    let mut results = Vec::new();

    for &count in account_counts {
        println!("Benchmarking MDBX proof generation time with {} accounts", count);

        // Create temporary directory for database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path();

        // Generate accounts and post state
        let post_state = generate_test_post_state(count);

        // Create a sampling of accounts to generate proofs for (about 10% of total)
        let sample_count = std::cmp::max(1, count / 10);
        let mut sample_addresses = Vec::with_capacity(sample_count);
        for i in 0..sample_count {
            let idx = (i * count / sample_count) % count;
            let mut addr_bytes = [0u8; 20];
            addr_bytes[0..8].copy_from_slice(&(idx as u64).to_be_bytes());
            sample_addresses.push(Address::from(addr_bytes));
        }

        // First populate the database with accounts
        {
            // Create the database with proper arguments
            let db = mdbx::init_db(
                db_path,
                mdbx::DatabaseArguments::new(reth_db_api::models::ClientVersion::default()),
            )
            .unwrap();

            // Get a transaction for updates
            let tx_mut = db.tx_mut().unwrap();

            // Convert post state to a format suitable for the trie
            let post_state_clone = post_state.clone();
            let prefix_sets = post_state_clone.construct_prefix_sets();
            let frozen_sets = prefix_sets.freeze();
            let state_sorted = post_state_clone.into_sorted();

            // Calculate state root with updates (this will update the trie)
            let (root, _updates) = reth_trie::StateRoot::new(
                reth_trie_db::DatabaseTrieCursorFactory::new(&tx_mut),
                reth_trie::hashed_cursor::HashedPostStateCursorFactory::new(
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

        // Now benchmark proof generation time
        for i in 0..iterations {
            println!("  Iteration {}/{}", i + 1, iterations);

            // Open the database for reading
            let db = mdbx::open_db(
                db_path,
                mdbx::DatabaseArguments::new(reth_db_api::models::ClientVersion::default()),
            )
            .unwrap();

            // Get a transaction for reading
            let tx = db.tx().unwrap();

            // Measure proof generation time
            let start = Instant::now();

            // Generate proofs for sample addresses
            for address in &sample_addresses {
                // Create proof generator for each address
                let proof_generator = Proof::new(
                    reth_trie_db::DatabaseTrieCursorFactory::new(&tx),
                    reth_trie_db::DatabaseHashedCursorFactory::new(&tx),
                );

                let _proof = proof_generator.account_proof(*address, &[]).unwrap();
                // Verify the proof contains data
                assert!(!_proof.proof.is_empty(), "Proof should not be empty");
            }

            let duration = start.elapsed();
            durations.push(duration);

            println!("    Completed in {:?}", duration);
        }

        // Calculate average duration
        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / iterations as u32;

        results.push((count, avg_duration));
        println!("Average proof generation time for {} accounts: {:?}", count, avg_duration);
    }

    Ok(results)
}

/// Benchmark MPT proof generation time for RocksDB implementation
pub fn benchmark_rocksdb_proof_gen_time(
    account_counts: &[usize],
    iterations: usize,
) -> Result<Vec<(usize, Duration)>> {
    let mut results = Vec::new();

    for &count in account_counts {
        println!("Benchmarking RocksDB proof generation time with {} accounts", count);

        // Create RocksDB database
        let (db, _temp_dir) = create_test_db();

        // Generate accounts and post state
        let post_state = generate_test_post_state(count);

        // Sample some addresses for proof generation (about 10% of total)
        let sample_count = std::cmp::max(1, count / 10);
        let mut sample_addresses = Vec::with_capacity(sample_count);
        for i in 0..sample_count {
            let idx = (i * count / sample_count) % count;
            let mut addr_bytes = [0u8; 20];
            addr_bytes[0..8].copy_from_slice(&(idx as u64).to_be_bytes());
            sample_addresses.push(Address::from(addr_bytes));
        }

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

        // Now benchmark proof generation time
        for i in 0..iterations {
            println!("  Iteration {}/{}", i + 1, iterations);

            // Create a read-only transaction
            let read_tx = RocksTransaction::<false>::new(db.clone(), false);

            // Create proof generator using the transaction's cursor factories

            // Measure proof generation time
            let start = Instant::now();

            // Generate proofs for sample addresses
            for address in &sample_addresses {
                let proof_generator = reth_trie::proof::Proof::new(
                    read_tx.trie_cursor_factory(),
                    read_tx.hashed_cursor_factory(),
                );
                let _proof = proof_generator.account_proof(*address, &[]).unwrap();
            }

            let duration = start.elapsed();
            durations.push(duration);

            println!("    Completed in {:?}", duration);
        }

        // Calculate average duration
        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / iterations as u32;

        results.push((count, avg_duration));
        println!("Average proof generation time for {} accounts: {:?}", count, avg_duration);
    }

    Ok(results)
}
