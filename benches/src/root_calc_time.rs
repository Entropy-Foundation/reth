// use anyhow::Result;
// use rand::prelude::StdRng;
// use rand::{Rng, SeedableRng};
// use reth_db::mdbx::{MdbxEnvironment, MdbxTransaction};
// use reth_primitives::{Address, Bytes, B256, H256, U256};
// use reth_trie::{
//     account::AccountTrie, db::mdbx::MdbxTrieStorage, AccountTrieMut, HashedAccount, HashedStorage,
//     Nibbles, StateRoot, StorageTrieMut, TrieStorage,
// };
// use std::time::Instant;
// use tempfile::TempDir;

// /// Generate a random address
// fn random_address(rng: &mut StdRng) -> Address {
//     let mut bytes = [0u8; 20];
//     rng.fill(&mut bytes[..]);
//     Address::from(bytes)
// }

// /// Generate a random B256 hash
// fn random_hash(rng: &mut StdRng) -> B256 {
//     let mut bytes = [0u8; 32];
//     rng.fill(&mut bytes[..]);
//     B256::from(bytes)
// }

// /// Measure MPT root calculation time for MDBX implementation
// pub fn benchmark_mdbx_root_calc_time(
//     account_counts: &[usize],
//     iterations: usize,
// ) -> Result<Vec<(usize, f64)>> {
//     let mut results = Vec::new();

//     for &count in account_counts {
//         println!("Benchmarking MDBX root calculation time with {} accounts", count);

//         // Create temporary directory for database
//         let tmp_dir = TempDir::new()?;
//         let db_path = tmp_dir.path();

//         // Initialize MDBX environment
//         let env = MdbxEnvironment::open(db_path, None)?;

//         // Create transaction and populate trie
//         {
//             let txn = env.begin_mutable_txn()?;
//             let mut db = MdbxTrieStorage::new(txn);

//             // Generate random accounts
//             let mut rng = StdRng::seed_from_u64(42); // Fixed seed for reproducibility
//             let accounts: Vec<_> = (0..count)
//                 .map(|_| {
//                     let address = random_address(&mut rng);
//                     let balance = U256::from(rng.gen_range(0..100000));
//                     let nonce = rng.gen_range(0..1000);
//                     let code_hash = random_hash(&mut rng);

//                     (address, HashedAccount { balance, nonce, bytecode_hash: Some(code_hash) })
//                 })
//                 .collect();

//             // Create account trie and insert accounts
//             let mut trie = AccountTrie::new(db.account_storage());
//             for (address, account) in accounts {
//                 trie.insert(address, account)?;
//             }

//             // Commit changes
//             db.commit()?;
//         }

//         // Measure root calculation time across multiple iterations
//         let mut total_duration = 0.0;

//         for i in 0..iterations {
//             println!("  Iteration {}/{}", i + 1, iterations);

//             // Create read transaction
//             let txn = env.begin_txn()?;
//             let db = MdbxTrieStorage::new(txn);

//             // Measure root calculation time
//             let start = Instant::now();

//             // Calculate state root
//             let state_root = db.state_root()?;

//             let duration = start.elapsed();
//             total_duration += duration.as_secs_f64();

//             println!("    Completed in {:.2}s, Root: {:?}", duration.as_secs_f64(), state_root);
//         }

//         let avg_duration = total_duration / iterations as f64;
//         results.push((count, avg_duration));

//         println!("Average root calculation time for {} accounts: {:.2}s", count, avg_duration);
//     }

//     Ok(results)
// }

// /// Measure MPT root calculation time for RocksDB implementation
// pub fn benchmark_rocksdb_root_calc_time(
//     account_counts: &[usize],
//     iterations: usize,
// ) -> Result<Vec<(usize, f64)>> {
//     // Note: This is a placeholder for the RocksDB implementation
//     // The actual implementation would be similar to the MDBX version but using RocksDB
//     println!("RocksDB benchmark - implementation will be similar to MDBX but with RocksDB backend");

//     // This would be replaced with actual RocksDB implementation
//     let results = account_counts.iter().map(|&count| (count, 0.0)).collect();
//     Ok(results)
// }
