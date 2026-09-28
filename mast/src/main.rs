#[derive(Clone)]
struct Api;

impl AsRef<Api> for Api {
    fn as_ref(&self) -> &Api {
        self
    }
}

#[tokio::main]
async fn main() {
    let router = openapi::server::new::<_, Api, ()>(Api);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    axum::serve(listener, router).await.unwrap();
}
