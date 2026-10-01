mod base64url;
mod board;
mod deflate;
mod room;
mod uuid_v4;

#[derive(Clone, Default)]
struct ApiImpl {
    board_events: std::sync::Arc<std::sync::Mutex<Vec<board::board_created::BoardCreated>>>,
    room_events: std::sync::Arc<std::sync::Mutex<Vec<room::room_created::RoomCreated>>>,
}

impl AsRef<ApiImpl> for ApiImpl {
    fn as_ref(&self) -> &ApiImpl {
        self
    }
}

impl openapi::apis::ErrorHandler<String> for ApiImpl {}

#[tokio::main]
async fn main() {
    let router = openapi::server::new::<_, ApiImpl, String>(ApiImpl::default());

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    axum::serve(listener, router).await.unwrap();
}
