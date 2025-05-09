use alloy_primitives::keccak256;
use alloy_primitives::{Address, B256, U256};
use anyhow::Result;
use reth_db::mdbx::{
    DatabaseFlags, Environment, EnvironmentFlags, Geometry, Mode, SyncMode, WriteFlags,
};
use reth_primitives::Account;
use reth_trie::HashedPostState;
use std::{
    ops::RangeInclusive,
    time::{Duration, Instant},
};
use tempfile::TempDir;

/// Generate random accounts for testing
pub fn generate_test_accounts(count: usize) -> Account {
    Account {
        nonce: count as u64,
        balance: U256::from(count * 1000),
        bytecode_hash: Some(B256::from([count as u8; 32])),
    }
}

/// Generate a test post state with the specified number of accounts
pub fn generate_test_post_state(count: usize) -> HashedPostState {
    let mut post_state = HashedPostState::default();

    for i in 0..count {
        // Create an address using a deterministic pattern
        let mut addr_bytes = [0u8; 20];
        addr_bytes[0..8].copy_from_slice(&(i as u64).to_be_bytes());
        let address = Address::from(addr_bytes);
        let hashed_address = keccak256(address);

        let account = generate_test_accounts(i);

        // Add account to post state
        post_state.accounts.insert(hashed_address, Some(account));

        // Add some storage for the account
        let mut storage = reth_trie::HashedStorage::default();

        // Add a few storage slots per account
        for j in 0..5 {
            let mut slot_bytes = [0u8; 32];
            slot_bytes[0..8].copy_from_slice(&(j as u64).to_be_bytes());
            let slot = B256::from(slot_bytes);
            let hashed_slot = keccak256(slot);

            storage.storage.insert(hashed_slot, U256::from(j * 100));
        }

        post_state.storages.insert(hashed_address, storage);
    }

    post_state
}

/// Helper function to set up MDBX environment
pub fn setup_mdbx_env(path: &std::path::Path) -> Result<Environment> {
    let env = Environment::builder()
        .set_max_dbs(10)
        .set_geometry(Geometry::<RangeInclusive<usize>> {
            size: Some(0..=8 * 1024 * 1024 * 1024), // Max 8GB
            ..Default::default()
        })
        .set_flags(EnvironmentFlags {
            mode: Mode::ReadWrite { sync_mode: SyncMode::UtterlyNoSync },
            no_rdahead: true,
            coalesce: true,
            ..Default::default()
        })
        .open(path)?;

    Ok(env)
}

/// Measure MPT update time for MDBX implementation
pub fn benchmark_mdbx_update_time(
    account_counts: &[usize],
    iterations: usize,
) -> Result<Vec<(usize, Duration)>> {
    let mut results = Vec::new();

    for &count in account_counts {
        println!("Benchmarking MDBX update time with {} accounts", count);

        let mut total_duration = Duration::from_secs(0);

        for i in 0..iterations {
            println!("  Iteration {}/{}", i + 1, iterations);

            // Create a temporary directory for the database and setup the environment
            let temp_dir = TempDir::new()?;
            let env = setup_mdbx_env(temp_dir.path())?;

            // Generate addresses and accounts
            let accounts: Vec<(Address, Account)> = (0..count)
                .map(|i| {
                    let mut addr_bytes = [0u8; 20];
                    for j in 0..std::cmp::min(8, std::mem::size_of::<usize>()) {
                        addr_bytes[j] = ((i >> (j * 8)) & 0xFF) as u8;
                    }
                    let address = Address::from_slice(&addr_bytes);
                    let account = generate_test_accounts(i);
                    (address, account)
                })
                .collect();

            // ToDo:> insert all the accounts into the database | wait till it gets updated

            let txn = env.begin_rw_txn()?;
            let db = txn.create_db(Some("accounts"), DatabaseFlags::empty())?;

            // Insert accounts
            for (idx, (_, account)) in accounts.iter().enumerate() {
                /*
                    This is a simplified approach for the benchmark. In the actual Reth implementation:
                        - Ethereum state is stored in a Merkle Patricia Trie (MPT)
                        - The typical key would be the keccak256 hash of the address
                        - The value would contain the full serialized account data (nonce, balance, storage root, code hash)

                    In the current code:
                        - Keys are formatted as "account:{index}" (e.g., "account:0", "account:1")
                        - Values are only storing the account nonce (not the full account data)
                */
                let key = format!("account:{}", idx).into_bytes();
                let mut value = Vec::new();
                value.extend_from_slice(&account.nonce.to_le_bytes());
                txn.put(db.dbi(), &key, &value, WriteFlags::empty())?;
            }
            txn.commit()?;

            // Measure update time
            let start = Instant::now();

            let txn = env.begin_rw_txn()?;
            // Update accounts
            for (idx, (_, account)) in accounts.iter().enumerate() {
                let key = format!("account:{}", idx).into_bytes();

                // Create a modified account (e.g., increment nonce or balance)
                let mut updated_account = account.clone();
                updated_account.balance += U256::from(1);
                // Serialize the updated account
                let mut value = Vec::new();
                value.extend_from_slice(&updated_account.nonce.to_le_bytes());

                txn.put(db.dbi(), &key, &value, WriteFlags::empty())?;
            }
            txn.commit()?;

            let duration = start.elapsed();
            total_duration += duration;

            println!("    Completed in {:.2}s", duration.as_secs_f64());
        }

        let avg_duration = total_duration / iterations as u32;
        results.push((count, avg_duration));

        println!("Average update time for {} accounts: {:.?}s", count, avg_duration);
    }

    Ok(results)
}

/// Measure MPT update time for RocksDB implementation
pub fn benchmark_rocksdb_update_time(
    account_counts: &[usize],
    _iterations: usize,
) -> Result<Vec<(usize, Duration)>> {
    // Note: This is a placeholder for the RocksDB implementation
    // The actual implementation would be similar to the MDBX version but using RocksDB
    println!("RocksDB benchmark - implementation will be similar to MDBX but with RocksDB backend");

    // This would be replaced with actual RocksDB implementation
    let results = account_counts.iter().map(|&count| (count, Duration::from_secs(0))).collect();
    Ok(results)
}

/*
/// Benchmark MPT update time using RocksDB implementation
pub fn benchmark_rocksdb_update_time(account_counts: &[usize], iterations: usize) -> Vec<(usize, Duration)> {
    let mut results = Vec::new();

    for &count in account_counts {
        println!("Benchmarking RocksDB update time with {} accounts", count);

        let mut total_duration = Duration::from_secs(0);

        for i in 0..iterations {
            println!("  Iteration {}/{}", i + 1, iterations);

            // Create temporary directory for database
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path();

            // Generate accounts and post state
            let post_state = generate_test_post_state(count);

            // Create RocksDB implementation
            let db = reth_db::test_utils::create_test_db_at_path(db_path).0;

            // Measure update time
            let start = Instant::now();

            // Create read and write transactions for RocksDB
            let read_tx = implementation::rocks::tx::RocksTransaction::<false>::new(db.clone(), false);
            let write_tx = implementation::rocks::tx::RocksTransaction::<true>::new(db.clone(), true);

            // Calculate state root with updates (this will update the trie)
            let _state_root = implementation::rocks::calculate_state_root_with_updates(
                &read_tx,
                &write_tx,
                post_state
            ).unwrap();

            // Commit transaction
            write_tx.commit().unwrap();

            let duration = start.elapsed();
            total_duration += duration;

            println!("    Completed in {:?}", duration);
        }

        let avg_duration = total_duration / iterations as u32;
        results.push((count, avg_duration));

        println!("Average update time for {} accounts: {:?}", count, avg_duration);
    }

    results
}

/// Generate a test post state with the specified number of accounts
fn generate_test_post_state(count: usize) -> HashedPostState {
    let mut post_state = HashedPostState::default();

    for i in 0..count {
        // Create an address using a deterministic pattern
        let mut addr_bytes = [0u8; 20];
        addr_bytes[0..8].copy_from_slice(&(i as u64).to_be_bytes());
        let address = Address::from(addr_bytes);
        let hashed_address = keccak256(address);

        // Create a test account
        let hashed_account = reth_trie::HashedAccount {
            balance: U256::from(i * 1000),
            nonce: (i % 1000) as u64,
            bytecode_hash: Some(B256::random()),
        };

        // Add account to post state
        post_state.accounts.insert(hashed_address, Some(hashed_account));

        // Add some storage for the account
        let mut storage = reth_trie::HashedStorage::default();

        // Add a few storage slots per account
        for j in 0..5 {
            let mut slot_bytes = [0u8; 32];
            slot_bytes[0..8].copy_from_slice(&(j as u64).to_be_bytes());
            let slot = B256::from(slot_bytes);
            let hashed_slot = keccak256(slot);

            storage.storage.insert(hashed_slot, U256::from(j * 100));
        }

        post_state.storages.insert(hashed_address, storage);
    }

    post_state
}
*/
