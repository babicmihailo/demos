use axum::{
    extract::{Path, State},
    http::{StatusCode, HeaderValue},
    response::Json,
    routing::{get, post},
    Router,
};
use prost::Message;
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{Error as IoError, ErrorKind};
use std::sync::{Arc, Mutex};
use tower_http::cors::{CorsLayer, Any};

mod protos;
use protos::redis_demo::{CreditWallet, Genre, SubscriptionLevel, UserProfile};

type SharedState = Arc<Mutex<redis::Connection>>;

#[derive(Serialize, Deserialize)]
struct GenreJson {
    id: String,
    name: String,
    listeners: i32,
}

#[derive(Serialize, Deserialize)]
struct UserProfileJson {
    id: String,
    username: String,
    email: String,
    subscription_level: i32,
}

#[derive(Serialize, Deserialize)]
struct WalletJson {
    coin_balance: i32,
    credit_balance: i32,
}

#[derive(Deserialize)]
struct TransferRequest {
    amount: i32,
}

fn decode_protobuf<T: Message + Default>(bytes: Vec<u8>) -> Result<T, Box<dyn Error>> {
    T::decode(&bytes[..]).map_err(|e| {
        Box::new(IoError::new(
            ErrorKind::InvalidData,
            format!("Protobuf Decode failed: {}", e),
        )) as Box<dyn Error>
    })
}

fn create_genre(con: &mut impl Commands, genre: Genre) -> Result<Genre, Box<dyn Error>> {
    let key = format!("genre:{}:metadata", genre.id);
    let index_key = "genres:all_ids";
    let mut buf = Vec::new();
    genre.encode(&mut buf)?;
    let _: () = con.set(&key, buf)?;
    let _: () = con.sadd(index_key, &genre.id)?;
    read_genre(con, &genre.id)
}

fn read_genre(con: &mut impl Commands, genre_id: &str) -> Result<Genre, Box<dyn Error>> {
    let key = format!("genre:{}:metadata", genre_id);
    let bytes: Vec<u8> = con.get(&key)?;
    decode_protobuf(bytes)
}

fn read_all_genres(con: &mut impl Commands) -> Result<Vec<Genre>, Box<dyn Error>> {
    let index_key = "genres:all_ids";
    let genre_ids: Vec<String> = con.smembers(index_key)?;
    let mut genres = Vec::with_capacity(genre_ids.len());
    for id in genre_ids {
        if let Ok(genre) = read_genre(con, &id) {
            genres.push(genre);
        }
    }
    Ok(genres)
}

fn create_profile(con: &mut impl Commands, profile: UserProfile) -> Result<UserProfile, Box<dyn Error>> {
    let key = format!("user:{}:profile", profile.id);
    let mut buf = Vec::new();
    profile.encode(&mut buf)?;
    let _: () = con.set(&key, buf)?;
    read_profile(con, &profile.id)
}

fn read_profile(con: &mut impl Commands, user_id: &str) -> Result<UserProfile, Box<dyn Error>> {
    let key = format!("user:{}:profile", user_id);
    let bytes: Vec<u8> = con.get(&key)?;
    decode_protobuf(bytes)
}

fn update_profile(con: &mut impl Commands, profile: UserProfile) -> Result<UserProfile, Box<dyn Error>> {
    let key = format!("user:{}:profile", profile.id);
    let mut buf = Vec::new();
    profile.encode(&mut buf)?;
    let _: () = con.set(&key, buf)?;
    Ok(profile)
}

fn read_wallet(con: &mut impl Commands, user_id: &str) -> Result<CreditWallet, Box<dyn Error>> {
    let key = format!("user:{}:wallet", user_id);
    let bytes: Vec<u8> = con.get(&key)?;
    decode_protobuf(bytes)
}

fn create_wallet(con: &mut impl Commands, user_id: &str) -> Result<CreditWallet, Box<dyn Error>> {
    let key = format!("user:{}:wallet", user_id);
    // Initial balance: 100 coins and 0 credits for new users
    let wallet = CreditWallet {
        coin_balance: 100,
        credit_balance: 0,
    };
    let mut buf = Vec::new();
    wallet.encode(&mut buf)?;
    let _: () = con.set(&key, buf)?;
    Ok(wallet)
}

fn transfer_credit_transaction(
    con: &mut impl Commands,
    user_id: &str,
    transfer_amount: i32,
) -> Result<CreditWallet, Box<dyn Error>> {
    if transfer_amount <= 0 {
        return Err(Box::new(IoError::new(
            ErrorKind::InvalidInput,
            "Transfer amount must be positive",
        )));
    }

    let balance_key = format!("user:{}:wallet", user_id);
    let final_wallet: CreditWallet = redis::transaction(con, &[&balance_key], |con, pipe| {
        let bytes: Vec<u8> = con.get(&balance_key)?;
        let mut current: CreditWallet = match decode_protobuf(bytes) {
            Ok(b) => b,
            Err(e) => {
                return Err(IoError::new(
                    ErrorKind::InvalidData,
                    format!("Decode error in transaction: {}", e),
                ).into());
            }
        };

        if current.coin_balance < transfer_amount {
            return Err(IoError::new(
                ErrorKind::InvalidInput,
                format!("Insufficient coins. Current balance: {}, requested: {}",
                        current.coin_balance, transfer_amount),
            ).into());
        }

        current.coin_balance -= transfer_amount;
        current.credit_balance += transfer_amount;
        let mut new_buf = Vec::new();
        if current.encode(&mut new_buf).is_err() {
            return Err(IoError::new(
                ErrorKind::InvalidData,
                "Protobuf Encode failed in transaction",
            ).into());
        }
        pipe.set(&balance_key, new_buf).ignore().query::<()>(con)?;
        Ok(Some(current))
    })?;
    Ok(final_wallet)
}

async fn get_all_genres(State(state): State<SharedState>) -> Result<Json<Vec<GenreJson>>, StatusCode> {
    let mut con = state.lock().unwrap();
    match read_all_genres(&mut *con) {
        Ok(genres) => {
            let json_genres: Vec<GenreJson> = genres.into_iter().map(|g| GenreJson {
                id: g.id,
                name: g.name,
                listeners: g.listeners,
            }).collect();
            Ok(Json(json_genres))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_genre_handler(
    State(state): State<SharedState>,
    Json(payload): Json<GenreJson>,
) -> Result<Json<GenreJson>, StatusCode> {
    let mut con = state.lock().unwrap();
    let genre = Genre {
        id: payload.id,
        name: payload.name,
        listeners: payload.listeners,
    };
    match create_genre(&mut *con, genre) {
        Ok(g) => Ok(Json(GenreJson { id: g.id, name: g.name, listeners: g.listeners })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_user_profile(
    State(state): State<SharedState>,
    Path(user_id): Path<String>,
) -> Result<Json<UserProfileJson>, StatusCode> {
    let mut con = state.lock().unwrap();
    match read_profile(&mut *con, &user_id) {
        Ok(profile) => Ok(Json(UserProfileJson {
            id: profile.id,
            username: profile.username,
            email: profile.email,
            subscription_level: profile.subscription_level,
        })),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_user_profile(
    State(state): State<SharedState>,
    Json(payload): Json<UserProfileJson>,
) -> Result<Json<UserProfileJson>, StatusCode> {
    let mut con = state.lock().unwrap();
    let profile = UserProfile {
        id: payload.id.clone(),
        username: payload.username,
        email: payload.email,
        subscription_level: payload.subscription_level,
        history_key: format!("{}:history", payload.id),
    };

    match create_profile(&mut *con, profile) {
        Ok(p) => {
            // Create initial wallet with starting balance
            if let Err(_) = create_wallet(&mut *con, &p.id) {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            Ok(Json(UserProfileJson {
                id: p.id,
                username: p.username,
                email: p.email,
                subscription_level: p.subscription_level,
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_user_profile(
    State(state): State<SharedState>,
    Path(user_id): Path<String>,
    Json(payload): Json<UserProfileJson>,
) -> Result<Json<UserProfileJson>, StatusCode> {
    let mut con = state.lock().unwrap();
    let profile = UserProfile {
        id: user_id.clone(),
        username: payload.username,
        email: payload.email,
        subscription_level: payload.subscription_level,
        history_key: format!("{}:history", user_id),
    };
    match update_profile(&mut *con, profile) {
        Ok(p) => Ok(Json(UserProfileJson {
            id: p.id,
            username: p.username,
            email: p.email,
            subscription_level: p.subscription_level,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_wallet(
    State(state): State<SharedState>,
    Path(user_id): Path<String>,
) -> Result<Json<WalletJson>, StatusCode> {
    let mut con = state.lock().unwrap();
    match read_wallet(&mut *con, &user_id) {
        Ok(wallet) => Ok(Json(WalletJson {
            coin_balance: wallet.coin_balance,
            credit_balance: wallet.credit_balance,
        })),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn transfer_credits(
    State(state): State<SharedState>,
    Path(user_id): Path<String>,
    Json(payload): Json<TransferRequest>,
) -> Result<Json<WalletJson>, StatusCode> {
    let mut con = state.lock().unwrap();
    match transfer_credit_transaction(&mut *con, &user_id, payload.amount) {
        Ok(wallet) => Ok(Json(WalletJson {
            coin_balance: wallet.coin_balance,
            credit_balance: wallet.credit_balance,
        })),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Insufficient coins") || error_msg.contains("must be positive") {
                Err(StatusCode::BAD_REQUEST)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let con = client.get_connection()?;
    let shared_state = Arc::new(Mutex::new(con));

    println!("Connected to Redis!");

    {
        let mut con = shared_state.lock().unwrap();
        let user_id = "user:1234";

        let pre_built_genres = vec![
            Genre { id: "ROCK".to_string(), name: "Classic Rock".to_string(), listeners: 8000000 },
            Genre { id: "POP".to_string(), name: "Global Pop Hits".to_string(), listeners: 15000000 },
            Genre { id: "JAZZ".to_string(), name: "Smooth Jazz".to_string(), listeners: 500000 },
        ];
        for genre in pre_built_genres {
            let _ = create_genre(&mut *con, genre);
        }

        let initial_profile = UserProfile {
            id: user_id.to_string(),
            username: "StarterUser".to_string(),
            email: "starter@example.com".to_string(),
            subscription_level: SubscriptionLevel::LevelFree.into(),
            history_key: format!("{}:history", user_id),
        };
        let _ = create_profile(&mut *con, initial_profile);

        let initial_wallet = CreditWallet { coin_balance: 100, credit_balance: 50 };
        let temp_wallet_key = format!("user:{}:wallet", user_id);
        let mut w_buf = Vec::new();
        initial_wallet.encode(&mut w_buf)?;
        let _: () = con.set(&temp_wallet_key, w_buf)?;

        println!("Initial data setup complete!");
    }

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/genres", get(get_all_genres).post(create_genre_handler))
        .route("/users/{user_id}", get(get_user_profile).put(update_user_profile))
        .route("/users", post(create_user_profile))
        .route("/users/{user_id}/wallet", get(get_wallet))
        .route("/users/{user_id}/wallet/transfer", post(transfer_credits))
        .layer(cors)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind to port 3001");

    println!("Server running on http://localhost:3001");

    if let Err(err) = axum::serve(listener, app).await {
        eprintln!("Server error: {err}");
    }

    Ok(())
}