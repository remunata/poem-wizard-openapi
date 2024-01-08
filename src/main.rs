use poem::{
    endpoint::StaticFilesEndpoint, listener::TcpListener, middleware::Cors, EndpointExt, Route,
};
use poem_openapi::OpenApiService;
use std::fs;

mod wizard_api;
mod wizard_responses;
mod wizard_service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "postgres://postgres:postgrespass@localhost/poem_wizard_api";
    let pool = sqlx::PgPool::connect(url).await?;

    sqlx::migrate!().run(&pool).await?;
    let _ = fs::create_dir("./files");

    let api_service = OpenApiService::new(wizard_api::WizardApi, "Wizard API", "1.0.0")
        .server("http://localhost:3000");

    let ui = api_service.openapi_explorer();

    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        .nest("/files", StaticFilesEndpoint::new("./files"))
        .with(Cors::new())
        .data(pool);

    poem::Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await?;

    Ok(())
}
