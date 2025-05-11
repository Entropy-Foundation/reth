use alloy_primitives::keccak256;
use alloy_primitives::{Address, B256, U256};
use anyhow::{Ok, Result};
use reth_db::transaction::{DbTx, DbTxMut};
use reth_db::{Database, HashedAccounts};
use reth_db_rocks::{utils::create_test_db, RocksTransaction};
use reth_primitives::Account;
use reth_trie::hashed_cursor::HashedPostStateCursorFactory;
use reth_trie::HashedPostState;
use std::time::{Duration, Instant};
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

/// Measure MPT update time for MDBX implementation
pub fn benchmark_mdbx_update_time(
    account_counts: &[usize],
    iterations: usize,
) -> Result<Vec<(usize, Duration)>> {
    let mut results = Vec::new();

    for &count in account_counts {
        println!("Benchmarking MDBX update time with {} accounts", count);

        // Create a temporary directory for the database (only once per account count)
        let temp_dir = TempDir::new()?;

        // Create the database with proper arguments (only once)
        let db = reth_db::mdbx::init_db(
            temp_dir.path(),
            reth_db::mdbx::DatabaseArguments::new(reth_db_api::models::ClientVersion::default()),
        )
        .unwrap();

        // Generate accounts once and reuse them
        let accounts: Vec<(Address, Account)> = (0..count)
            .map(|i| {
                let mut addr_bytes = [0u8; 20];
                addr_bytes[0..8].copy_from_slice(&(i as u64).to_be_bytes());
                let address = Address::from(addr_bytes);
                let account = generate_test_accounts(i);
                (address, account)
            })
            .collect();

        // Store initial state (only once)
        {
            let tx_mut = db.tx_mut()?;

            for (address, account) in &accounts {
                let hashed_address = keccak256(*address);
                tx_mut.put::<HashedAccounts>(hashed_address, account.clone())?;
            }

            tx_mut.inner.commit()?;
        }

        let mut total_duration = Duration::from_secs(0);

        for i in 0..iterations {
            println!("  Iteration {}/{}", i + 1, iterations);

            // Now measure just the update time
            let start = Instant::now();

            // Open transaction for updates
            let tx_mut = db.tx_mut()?;

            // Update accounts - just increment balance and save
            for (address, account) in &accounts {
                let mut updated_account = account.clone();
                updated_account.balance += U256::from(1); // Update the account
                let hashed_address = keccak256(*address);
                tx_mut.put::<HashedAccounts>(hashed_address, updated_account)?;
            }

            // Commit the updates
            tx_mut.inner.commit()?;

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
    iterations: usize,
) -> Result<Vec<(usize, Duration)>> {
    let mut results = Vec::new();

    for &count in account_counts {
        println!("Benchmarking RocksDB update time with {} accounts", count);

        let mut total_duration = Duration::from_secs(0);

        for i in 0..iterations {
            println!("  Iteration {}/{}", i + 1, iterations);

            // Create a temporary directory for the database and setup the environment
            let (db, _temp_dir) = create_test_db();

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

            // Create read and write transactions
            let _read_tx = RocksTransaction::<false>::new(db.clone(), false);
            let write_tx = RocksTransaction::<true>::new(db.clone(), true);

            for (idx, (_, account)) in accounts.iter().enumerate() {
                let mut addr_bytes = [0u8; 20];
                for j in 0..std::cmp::min(8, std::mem::size_of::<usize>()) {
                    addr_bytes[j] = ((idx >> (j * 8)) & 0xFF) as u8;
                }
                let address = Address::from_slice(&addr_bytes);
                let hashed_address = keccak256(address);
                write_tx.put::<HashedAccounts>(hashed_address, account.clone())?;
            }

            // Commit the transaction
            write_tx.commit()?;

            // Measure update time
            let start = Instant::now();

            // Create a new transaction for updates
            let write_tx = RocksTransaction::<true>::new(db.clone(), true);

            // Update accounts
            for (idx, (_, account)) in accounts.iter().enumerate() {
                // Create a modified account (increment balance)
                let mut updated_account = account.clone();
                updated_account.balance += U256::from(1);

                let mut addr_bytes = [0u8; 20];
                for j in 0..std::cmp::min(8, std::mem::size_of::<usize>()) {
                    addr_bytes[j] = ((idx >> (j * 8)) & 0xFF) as u8;
                }
                let address = Address::from_slice(&addr_bytes);
                let hashed_address = keccak256(address);
                write_tx.put::<HashedAccounts>(hashed_address, updated_account)?;
            }

            // Commit the transaction with updates
            write_tx.commit()?;

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
