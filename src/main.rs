use axum::{
    extract::Path,
    response::Redirect,
    routing::{get, post},
    Json, Router, http::StatusCode,
};
use nanoid::nanoid;
use serde::Deserialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
struct State {
    links: RwLock<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
struct ShortenUrlPayload {
    url: String,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(State {
        links: RwLock::new(HashMap::new()),
    });

    let app = Router::new()
        .route(
            "/shorten",
            post({
                let shared_state = Arc::clone(&shared_state);
                move |body| shorten_handler(body, shared_state)
            }),
        )
        .route(
            "/:path",
            get({
                let shared_state = Arc::clone(&shared_state);
                move |path: Path<String>| redirect_handler(path, shared_state)
            }),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 1029));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap_or_else(|e| eprintln!("Server failed to start: {e}"))
}

async fn shorten_handler(
    Json(payload): Json<ShortenUrlPayload>,
    state: Arc<State>,
) -> Result<String, String> {
    let mut links = state.links.write().map_err(|e| format!("{} Error with locks: {e}", StatusCode::INTERNAL_SERVER_ERROR))?;
    let mut short_uuid;
    loop {
        short_uuid = nanoid!(7);
        if !links.contains_key(&short_uuid) {
            break;
        }
    }
    links.insert(short_uuid.clone(), payload.url);
    Ok(format!(
        "Success - shortened URL: http://localhost:1029/{}",
        short_uuid
    ))
}

async fn redirect_handler(path: Path<String>, state: Arc<State>) -> Result<Redirect, String> {
    let links = state.links.write().map_err(|e| format!("{} Error with locks: {e}", StatusCode::INTERNAL_SERVER_ERROR))?;
    if let Some(url) = links.get(&path.0) {
        Ok(Redirect::permanent(url))
    } else {
        Err("Url not found".to_string())
    }
}
