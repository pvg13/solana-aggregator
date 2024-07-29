use serde::{Deserialize, Serialize};
use solana_sdk::{clock::UnixTimestamp, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::EncodedTransactionWithStatusMeta;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TransactionInfo {
    pub sender: String,
    pub receivers: Vec<String>,
    pub fee: u64,
    pub timestamp: UnixTimestamp,
    pub signatures: Vec<String>,
}

impl TransactionInfo {
    pub fn accounts(&self) -> Vec<Pubkey> {
        // Join the sender and receivers
        let mut vec: Vec<Pubkey> = self
            .receivers
            .iter()
            .map(|x| Pubkey::from_str(x.as_str()).unwrap())
            .clone()
            .collect();

        vec.push(Pubkey::from_str(self.sender.as_str()).unwrap());

        vec
    }
}

impl TransactionInfo {
    pub fn signatures(&self) -> Vec<Signature> {
        self.signatures
            .iter()
            .map(|x| Signature::from_str(x.as_str()).unwrap())
            .collect()
    }
}

impl TryFrom<EncodedTransactionWithStatusMeta> for TransactionInfo {
    type Error = String;

    fn try_from(value: EncodedTransactionWithStatusMeta) -> Result<Self, Self::Error> {
        let decoded_transaction = value.transaction.decode().ok_or("Error decoding")?;
        let message = decoded_transaction.message;

        let accounts: Vec<String> = message
            .static_account_keys()
            .iter()
            .map(|x| x.to_string())
            .collect();

        let (sender, receivers) = accounts.split_first().ok_or("Error splitting ")?;

        let signatures = decoded_transaction
            .signatures
            .iter()
            .map(|x| x.to_string())
            .collect();

        let fee = value.meta.unwrap().fee;

        Ok(Self {
            sender: sender.clone(),
            receivers: Vec::from(receivers),
            fee,
            timestamp: 0,
            signatures,
        })
    }
}
