mod routes;
mod services;
mod business_logic;

use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(routes::health::health),
    components(schemas(routes::health::HealthResponse))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(routes::health::health))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    println!("Swagger UI: http://localhost:3000/swagger-ui");
    axum::serve(listener, app).await.unwrap();
}
