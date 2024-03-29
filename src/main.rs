use poem::{
    endpoint::StaticFilesEndpoint, listener::TcpListener, middleware::Cors, EndpointExt, Route,
};
use poem_openapi::OpenApiService;
use std::fs;

mod wizard_api;
mod wizard_responses;
mod wizard_service;

fn database_url() -> String {
    let user = std::env::var("PG_USER").unwrap_or_else(|_| "postgres".into());
    let password = std::env::var("PG_PASSWORD").unwrap_or_else(|_| "postgrespass".into());
    let host = std::env::var("PG_HOST").unwrap_or_else(|_| "localhost".into());
    let dbname = std::env::var("PG_DBNAME").unwrap_or_else(|_| "poem_wizard_openapi".into());

    format!("postgres://{}:{}@{}/{}", user, password, host, dbname).to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = fs::create_dir("./files");

    let file_appender = tracing_appender::rolling::hourly("./files/log/", "wizard.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_thread_ids(true)
        .with_target(true)
        .with_writer(non_blocking)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let url = database_url();
    let pool = sqlx::PgPool::connect(&url).await?;

    sqlx::migrate!().run(&pool).await?;

    let api_service = OpenApiService::new(wizard_api::WizardApi, "Wizard API", "1.0.0");

    let ui = api_service.openapi_explorer();

    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        .nest("/files", StaticFilesEndpoint::new("./files"))
        .with(Cors::new())
        .data(pool);

    println!("Server starting at port 3000");
    poem::Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await?;

    Ok(())
}
