use poem::{listener::TcpListener, EndpointExt, Route};
use poem_openapi::OpenApiService;

mod wizard_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool =
        sqlx::PgPool::connect("postgres://postgres:postgrespass@localhost/poem_wizard_api").await?;

    let api_service = OpenApiService::new(wizard_api::WizardApi, "Wizard API", "1.0.0")
        .server("http://localhost:3000");

    let ui = api_service.swagger_ui();

    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        .data(pool);

    poem::Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await?;

    Ok(())
}
