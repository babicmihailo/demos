use prost::Message;
use redis::Commands;
use std::error::Error;
use std::io::{Error as IoError, ErrorKind};

mod protos;
use protos::redis_demo::{CreditWallet, ListenHistory, SubscriptionLevel, UserProfile, Genre};

// Helper function for consistent Protobuf decoding
fn decode_protobuf<T: Message + Default>(bytes: Vec<u8>) -> Result<T, Box<dyn Error>> {
    T::decode(&bytes[..]).map_err(|e| {
        Box::new(IoError::new(
            ErrorKind::InvalidData,
            format!("Protobuf Decode failed: {}", e),
        )) as Box<dyn Error>
    })
}

// --- REDIS LOGIC FUNCTIONS (CRUD) ---

// --- GENRE CRUD ---

/// POST /genres
fn create_genre(con: &mut impl Commands, genre: Genre) -> Result<Genre, Box<dyn Error>> {
    let key = format!("genre:{}:metadata", genre.id);
    let index_key = "genres:all_ids";

    let mut buf = Vec::new();
    genre.encode(&mut buf)?;

    let _: () = con.set(&key, buf)?;
    let _: () = con.sadd(index_key, &genre.id)?;

    read_genre(con, &genre.id)
}

/// GET /genres/{id}
fn read_genre(con: &mut impl Commands, genre_id: &str) -> Result<Genre, Box<dyn Error>> {
    let key = format!("genre:{}:metadata", genre_id);
    let bytes: Vec<u8> = con.get(&key)?;
    decode_protobuf(bytes)
}

/// GET /genres (Read All)
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

// --- USER PROFILE CRUD ---

/// POST /users
fn create_profile(
    con: &mut impl Commands,
    profile: UserProfile,
) -> Result<UserProfile, Box<dyn Error>> {
    let key = format!("user:{}:profile", profile.id);
    let mut buf = Vec::new();
    profile.encode(&mut buf)?;
    let _: () = con.set(&key, buf)?;
    read_profile(con, &profile.id)
}

/// GET /users/{id}
fn read_profile(con: &mut impl Commands, user_id: &str) -> Result<UserProfile, Box<dyn Error>> {
    let key = format!("user:{}:profile", user_id);
    let bytes: Vec<u8> = con.get(&key)?;
    decode_protobuf(bytes)
}

/// PUT/PATCH /users/{id}
fn update_profile(
    con: &mut impl Commands,
    profile: UserProfile,
) -> Result<UserProfile, Box<dyn Error>> {
    let key = format!("user:{}:profile", profile.id);
    let mut buf = Vec::new();
    profile.encode(&mut buf)?;
    let _: () = con.set(&key, buf)?;
    Ok(profile)
}

// --- LISTEN HISTORY CRUD ---

/// POST /users/{id}/history (Stores/Overwrites history)
fn store_listen_history(
    con: &mut impl Commands,
    user_id: &str,
    history: ListenHistory,
) -> Result<ListenHistory, Box<dyn Error>> {
    let key = format!("user:{}:history", user_id);

    let mut buf = Vec::new();
    history.encode(&mut buf)?;
    let _: () = con.set(&key, buf)?;

    read_listen_history(con, user_id)
}

/// GET /users/{id}/history
fn read_listen_history(
    con: &mut impl Commands,
    user_id: &str,
) -> Result<ListenHistory, Box<dyn Error>> {
    let key = format!("user:{}:history", user_id);

    let bytes: Vec<u8> = con.get(&key)?;
    decode_protobuf(bytes)
}

// --- CREDIT WALLET / TRANSACTION ---

/// GET /users/{id}/wallet
fn read_wallet(con: &mut impl Commands, user_id: &str) -> Result<CreditWallet, Box<dyn Error>> {
    let key = format!("user:{}:wallet", user_id);

    let bytes: Vec<u8> = con.get(&key)?;
    decode_protobuf(bytes)
}

/// POST /users/{id}/wallet/transfer
fn transfer_credit_transaction(
    con: &mut impl Commands,
    user_id: &str,
    transfer_amount: i32,
) -> Result<CreditWallet, Box<dyn Error>> {
    let balance_key = format!("user:{}:wallet", user_id);

    let final_wallet: CreditWallet = redis::transaction(con, &[&balance_key], |con, pipe| {
        let bytes: Vec<u8> = con.get(&balance_key)?;

        let mut current: CreditWallet = match decode_protobuf(bytes) {
            Ok(b) => b,
            Err(e) => {
                return Err(IoError::new(
                    ErrorKind::InvalidData,
                    format!("Decode error in transaction: {}", e),
                )
                    .into());
            }
        };

        current.coin_balance -= transfer_amount;
        current.credit_balance += transfer_amount;

        let mut new_buf = Vec::new();
        if current.encode(&mut new_buf).is_err() {
            return Err(IoError::new(
                ErrorKind::InvalidData,
                "Protobuf Encode failed in transaction",
            )
                .into());
        }

        pipe.set(&balance_key, new_buf).ignore().query::<()>(con)?;

        Ok(Some(current))
    })?;

    Ok(final_wallet)
}

// --- MAIN EXECUTION ---

fn main() -> Result<(), Box<dyn Error>> {
    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_connection()?;
    println!("Connected to Redis! Demos V5 (Full CRUD).");

    // --- SETUP: Initial Data Creation ---
    // The main function sets up the required data state for the demos.
    let user_id = "user:1234";

    // Genre Setup
    println!("\n=== SETUP: Creating Genre Data ===");
    let pre_built_genres = vec![
        Genre { id: "ROCK".to_string(), name: "Classic Rock".to_string(), listeners: 8000000 },
        Genre { id: "POP".to_string(), name: "Global Pop Hits".to_string(), listeners: 15000000 },
        Genre { id: "JAZZ".to_string(), name: "Smooth Jazz".to_string(), listeners: 500000 },
    ];
    for genre in pre_built_genres {
        create_genre(&mut con, genre)?;
    }
    println!("3 Genres created.");

    // User Data Setup
    let initial_profile = UserProfile {
        id: user_id.to_string(),
        username: "StarterUser".to_string(),
        email: "starter@example.com".to_string(),
        subscription_level: SubscriptionLevel::LevelFree.into(),
        history_key: format!("{}:history", user_id),
    };
    let initial_wallet = CreditWallet { coin_balance: 100, credit_balance: 50 };
    let initial_history = ListenHistory { genres: vec![] };

    create_profile(&mut con, initial_profile)?;
    store_listen_history(&mut con, user_id, initial_history)?;
    let temp_wallet_key = format!("user:{}:wallet", user_id);
    let mut w_buf = Vec::new();
    initial_wallet.encode(&mut w_buf)?;
    let _: () = con.set(&temp_wallet_key, w_buf)?;
    println!("Initial User Data created for {}.", user_id);

    // --- DEMOS ---

    // 1. READ ALL GENRES DEMO
    println!("\n=== 1. READ ALL GENRES (GET /genres) ===");
    match read_all_genres(&mut con) {
        Ok(genres) => {
            println!("Found {} genres:", genres.len());
            for g in genres {
                println!("  -> {}: {} listeners", g.name, g.listeners);
            }
        },
        Err(e) => eprintln!("Read all genres failed: {}", e),
    }

    // 2. UPDATE PROFILE DEMO
    println!("\n=== 2. UPDATE PROFILE (PUT) ===");
    let updated_profile = UserProfile {
        username: "PremiumMaster".to_string(),
        subscription_level: SubscriptionLevel::LevelPremium.into(),
        ..read_profile(&mut con, user_id)?
    };
    match update_profile(&mut con, updated_profile) {
        Ok(p) => println!("Update: Profile changed to {} (Sub: {:?}).", p.username, SubscriptionLevel::try_from(p.subscription_level).unwrap()),
        Err(e) => eprintln!("Update profile failed: {}", e),
    }

    // 3. ATOMIC TRANSACTION DEMO
    println!("\n=== 3. ATOMIC TRANSFER (POST) ===");
    let transfer_amount = 20;
    println!("Initial Wallet: {:?}", read_wallet(&mut con, user_id)?);

    match transfer_credit_transaction(&mut con, user_id, transfer_amount) {
        Ok(wallet) => {
            println!("Transfer: Successful. Final Coins: {}, Credits: {}", wallet.coin_balance, wallet.credit_balance);
        },
        Err(e) => eprintln!("Transaction failed: {}", e),
    }

    // INFO
    println!("\n=== DATABASE INFO ===");
    let db_size: i32 = redis::cmd("DBSIZE").query(&mut con)?;
    println!("Keys in database: {}", db_size);

    println!("\nAll demos completed!");

    Ok(())
}