use axum::{
    routing::{get, post},
    Router,
    Json,
};
use serde::Deserialize;
mod tokenizer;
use tokenizer::frequency_counter_from_text;
mod db;
use db::{vocab_user, words};
use std::env;
use postgres::{Client, NoTls};
use std::collections::HashMap;
use serde_json::json;
use tokio::task;
use tower_http::cors::{CorsLayer, Any};

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let db_url = env::var("DATABASE_URL").unwrap();
    let mut client = Client::connect(&db_url, NoTls).unwrap();
    let init = db::init_tables(&mut client).unwrap();

    let app = Router::new()
        .route("/", get(|| async {"Hello, World!"}))
        .route("/analyze-text", post(analyse_text))
        .route("/toggle-word", post(toggle_word_knowledge))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct Text {
    user_id: i32,
    text: String,
    language: String,
}

async fn analyse_text(Json(payload): Json<Text>) -> Json<serde_json::Value> {
    let result = task::spawn_blocking(move || {
        let db_url = env::var("DATABASE_URL").unwrap();
        let mut client = Client::connect(&db_url, NoTls).unwrap();

        let freq = frequency_counter_from_text(&payload.text);
        let words: Vec<String> = freq.into_iter().map(|(word, _)| word).collect();
        let known_user_word_ids: Vec<i32> = vocab_user::get_words_for_user(&mut client, payload.user_id).unwrap();
        let known_user_words: Vec<String> = known_user_word_ids
            .into_iter()
            .filter_map(|id| words::get_text_word(&mut client, id).ok().flatten())
            .collect();
        let unknown_words: Vec<String> = words
            .iter()
            .filter(|word| !known_user_words.contains(word))
            .cloned()
            .collect();
        let known_words: Vec<String> = known_user_words
            .into_iter()
            .filter(|word| words.contains(word))
            .collect();

        let new_words = words::insert_multiple_words(&mut client, &unknown_words, &payload.language).unwrap();

        let mut word_status = HashMap::new();

        for word in known_words {
            word_status.insert(word, "known");
        }

        for word in unknown_words {
            word_status.entry(word).or_insert("unknown");
        }

        serde_json::json!({"words": word_status})
    }).await.unwrap();

    Json(result)
}

#[derive(Deserialize)]
struct Update {
    user_id: i32,
    word: String,
    language: String,
    status: String,
}

async fn toggle_word_knowledge(Json(payload): Json<Update>) -> Json<serde_json::Value>{
    let response_json = task::spawn_blocking(move || {
        let db_url = env::var("DATABASE_URL").unwrap();
        let mut client = Client::connect(&db_url, NoTls).unwrap();

        let word = payload.word;
        let status = payload.status;
        let language = payload.language;
        let user_id = payload.user_id;

        let word_id_opt = words::get_id_word(&mut client, &word, &language).unwrap();

        if let Some(word_id) = word_id_opt {
            match status.as_str() {
                "known" => {
                    vocab_user::insert_vocab_user(&mut client, user_id, word_id).unwrap();
                }
                "unknown" => {
                    vocab_user::delete_vocab_user(&mut client, user_id, word_id).unwrap();
                }
                _ => {
                    return json!({
                        "success": false,
                        "message": format!("Invalid status: {}", status)
                    });
                }
            }

            json!({
                "success": true,
                "message": format!("Word '{}' marked as {}", word, status),
                "word": word,
                "status": status
            })
        } else {
            json!({
                "success": false,
                "message": format!("Word '{}' not found in language '{}'", word, language)
            })
        }
    }).await.unwrap();

    Json(response_json)
}

