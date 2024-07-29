pub mod errors;
pub mod model;
pub mod workers;
// pub mod db;

use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use model::TransactionInfo;
use serde::Deserialize;
use solana_sdk::{account::Account, pubkey::Pubkey, signature::Signature};
use tokio::sync::RwLock;
use workers::fetch_worker;

#[derive(Deserialize)]
struct TransactionQuery {
    day: Option<String>,
    id: Option<String>,
    random: Option<u32>,
}

// Shared data struct holding the solana data
struct DataState {
    transactions: HashMap<Signature, Box<TransactionInfo>>,
    accounts: HashMap<Pubkey, Account>,
}

type AppState = Arc<RwLock<DataState>>;

// API handlers
async fn transaction(
    query: Query<TransactionQuery>,
    state: State<AppState>,
) -> Result<Json<Vec<TransactionInfo>>, ()> {
    match &query.id {
        Some(id) => {
            let reader = state.read().await;
            if let Some(transaction) = reader
                .transactions
                .get(&Signature::from_str(id.as_str()).unwrap())
            {
                return Ok(Json(vec![*transaction.clone()]));
            }
        }
        None => {}
    };

    match &query.day {
        Some(day) => {
            let reader = state.read().await;
            let transactions = reader
                .transactions
                .iter()
                .filter(|(_, v)| {
                    let naive_date = NaiveDate::parse_from_str(day, "%d/%m/%Y").unwrap();
                    let time = Utc
                        .from_utc_datetime(&naive_date.and_time(NaiveTime::default()))
                        .timestamp();
                    log::info!("Time: {:?}", time);
                    time <= v.timestamp
                        && v.timestamp < time + chrono::Duration::days(1).num_seconds()
                })
                .map(|(_, v)| *v.clone())
                .collect::<Vec<TransactionInfo>>();

            return Ok(Json(transactions));
        }
        None => {}
    }

    match &query.random {
        Some(latest) => {
            let reader = state.read().await;
            let transactions = reader
                .transactions
                .iter()
                .take(*latest as usize)
                .map(|(_, v)| *v.clone())
                .collect::<Vec<TransactionInfo>>();
            return Ok(Json(transactions));
        }
        None => {}
    }

    // Json(TransactionInfo::default())
    Err(())
}

async fn account(
    query: Query<TransactionQuery>,
    state: State<AppState>,
) -> Result<Json<Vec<Account>>, ()> {
    match &query.id {
        Some(id) => {
            let reader = state.read().await;
            if let Some(account) = reader.accounts.get(&Pubkey::from_str(id.as_str()).unwrap()) {
                return Ok(Json(vec![account.clone()]));
            }
        }
        None => {}
    };

    Ok(Json(vec![]))
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let state = Arc::new(RwLock::new(DataState {
        transactions: HashMap::new(),
        accounts: HashMap::new(),
    }));

    let rpc_url = "https://api.devnet.solana.com"; // Mainnet URL

    let worker = tokio::spawn(fetch_worker(rpc_url.to_string(), state.clone()));

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/transaction", get(transaction))
        .route("/account", get(account))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    // Wait for the worker to finish
    worker.await.unwrap();
}
