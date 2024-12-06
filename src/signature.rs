use crate::transaction::Transaction;

use hex;
use sha2::{Digest, Sha256};

pub fn verify_signature(transaction: &Transaction) -> bool {
    let hash_hex = sign_signature(transaction);

    // Compare the calculated hash with the transaction's signature
    hash_hex == transaction.signature
}

pub fn sign_signature(transaction: &Transaction) -> String {
    // Create a Sha256 object
    let mut hasher = Sha256::new();

    // Write the transaction data (id, nonce, payload) to the hasher
    hasher.update(&transaction.id);
    hasher.update(transaction.nonce.to_string());
    hasher.update(&transaction.payload);

    // Calculate the hash
    let result = hasher.finalize();

    // Convert the hash result to hex string
    hex::encode(result)
}
