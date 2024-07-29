use std::sync::Arc;

use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcBlockConfig};
use solana_sdk::{clock::UnixTimestamp, pubkey::Pubkey};
use solana_transaction_status::UiTransactionEncoding;

use tokio::{
    sync::RwLock,
    time::{sleep, Duration},
};

use crate::{model::TransactionInfo, DataState};

/**
 * Fetches and updates the accounts
 */
async fn fetch_accounts(
    rpc_url: String,
    pubkeys: Vec<Pubkey>,
    data: Arc<RwLock<DataState>>,
) -> Result<(), ()> {
    let client = RpcClient::new(rpc_url.clone());

    for pk in pubkeys {
        let account = match client.get_account(&pk).await {
            Ok(x) => x,
            Err(e) => {
                log::error!("Error fetching account: {e}");
                continue;
            }
        };

        let mut writer = data.write().await;
        writer.accounts.insert(pk.clone(), account);
    }

    Ok(())
}

/**
   Fetches the latest transactions and spawns a new worker to fetch the accounts
*/
async fn fetch_transaction_and_spawn_accounts(
    rpc_url: &String,
    current_slot: u64,
    state: &Arc<RwLock<DataState>>,
) -> Result<u64, ()> {
    let client = RpcClient::new(rpc_url.clone());

    // Get latest blocks
    let slots = client.get_blocks_with_limit(current_slot, 4).await.unwrap();

    let config = RpcBlockConfig {
        encoding: Some(UiTransactionEncoding::Base64),
        max_supported_transaction_version: Some(0),
        ..RpcBlockConfig::default()
    };

    log::info!("Slots: {:?}", slots);

    for slot in slots {
        let block = client.get_block_with_config(slot, config).await.unwrap();

        // log::info!("Block: {:?}", block);
        log::info!("Slot: {:?}", slot);

        let timestamp = block.block_time.unwrap();

        let enc_transactions = block.transactions.unwrap();

        for transaction_status in enc_transactions {
            let mut transition_decode = match TransactionInfo::try_from(transaction_status) {
                Ok(x) => x,
                Err(e) => {
                    log::error!("Error decoding transaction: {e}");
                    continue;
                }
            };

            transition_decode.timestamp = UnixTimestamp::from(timestamp);

            log::info!("Transaction time: {:?}", transition_decode.timestamp);

            let transaction = Box::new(transition_decode);

            log::info!("Transaction: {:?}", transaction.signatures[0]);

            // let signatures = transaction.signatures;s

            // log::info!("Transaction: {:?}", transaction.accounts());

            let pubkeys = transaction.accounts();

            let future = tokio::spawn(fetch_accounts(rpc_url.clone(), pubkeys, state.clone()));

            for signature in transaction.signatures() {
                state
                    .write()
                    .await
                    .transactions
                    .insert(signature.clone(), transaction.clone());
            }

            future.await.unwrap();
        }
    }

    Ok(0)
}

pub async fn fetch_worker(rpc_url: String, state: Arc<RwLock<DataState>>) {
    let client = RpcClient::new(rpc_url.clone());
    let last_slot = client.get_highest_snapshot_slot().await.unwrap().full;

    loop {
        _ = fetch_transaction_and_spawn_accounts(&rpc_url, last_slot, &state)
            .await
            .unwrap();
        // fetch_and_process_accounts(&client, &pubkeys).await;
        // log::info!("Account and transaction: {} - {}", accounts.len(), transactions.len());
        sleep(Duration::from_secs(60)).await; // Polling interval
    }
}
