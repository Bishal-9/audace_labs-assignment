mod network;
mod signature;
mod transaction;
mod utilities;

use network::{server, TransactionReceiver};
use signature::{sign_signature, verify_signature};
use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc::channel, Arc, RwLock},
    thread::scope,
};
use transaction::Transaction;
use utilities::{has_consecutive_nonces, convert_transactions_to_json};

fn main() {
    scope(|scoped_thread| {
        let (sender, receiver_channel) = channel::<String>();
        scoped_thread.spawn(move || {
            server(receiver_channel);
        });

        // Create a TransactionReceiver
        let receiver = Arc::new(RwLock::new(TransactionReceiver::new("0.0.0.0:5000")));
        let updated_nonces: Arc<RwLock<HashMap<String, u64>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let signatures: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(HashSet::new()));
        let pending_transactions: Arc<RwLock<HashMap<String, Vec<Transaction>>>> =
            Arc::new(RwLock::new(HashMap::new()));

        loop {
            let _receiver = Arc::clone(&receiver);
            let _updated_nonces = Arc::clone(&updated_nonces);
            let _signatures = Arc::clone(&signatures);
            let _pending_transactions = Arc::clone(&pending_transactions);
            let server_channel = sender.clone();

            if let Some(transaction) = receiver.write().unwrap().receive_transaction() {
                scoped_thread.spawn(move || {

                    // Check whether the last nonce is less for every wallet address
                    let last_nonce = match _updated_nonces.read().unwrap().get(&transaction.id) {
                        Some(&nonce) => nonce,
                        None => 0,
                    };
                    if last_nonce < transaction.nonce {
                        _updated_nonces
                            .write()
                            .unwrap()
                            .insert(transaction.id.clone(), transaction.nonce);
                    }

                    let signature = sign_signature(&transaction);

                    //  Verifying Signature
                    if !verify_signature(&transaction) {
                        println!("Transaction: {:?}", transaction);
                        println!("Signature NOT OK: {}", signature);
                        return;
                    }

                    // If same signature already exist then ignore otherwise store them for processing
                    if !_signatures.read().unwrap().contains(&signature) {
                        _signatures.write().unwrap().insert(signature);
                        let mut _pending_transactions = _pending_transactions.write().unwrap();

                        let key = transaction.id.clone();
                        let updated_transactions = _pending_transactions
                            .entry(key.clone())
                            .or_insert_with(Vec::new);
                        updated_transactions.push(transaction.clone());
                        
                        if updated_transactions.len() >= 5 {
                            
                            // Sort the HashSet according to nonce
                            updated_transactions.sort_by_key(|tx| tx.nonce);
                            
                            // Check for consecutive nonces
                            let is_consecutive = has_consecutive_nonces(&updated_transactions);
                            
                            // Check whether it is starting from the last updated nonce
                            if last_nonce + 1 != updated_transactions[0].nonce && !is_consecutive {
                                return;
                            }
                            
                            // Send batch data
                            let data_to_send = convert_transactions_to_json(&updated_transactions);
                            // println!("{}", data_to_send.to_string());
                            
                            // Send the transaction data to the channel
                            if let Err(e) = server_channel.send(format!("{:?}", data_to_send.to_string())) {
                                eprintln!("Failed to send transaction: {}", e);
                            }
                            
                            // Flush all transactions
                            updated_transactions.drain(0..5);
                            let txs = updated_transactions.to_vec();
                            _pending_transactions
                                .insert(key, txs);
                        }
                    }
                });
            }
        }
    });
}
