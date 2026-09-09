use std::process::ExitCode;
use std::sync::Arc;

#[tokio::main]
async fn main() -> ExitCode {
    // Structured logs; tune with RUST_LOG (e.g. RUST_LOG=starter=debug).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Local configuration from .env, if present; real environment wins.
    dotenvy::dotenv().ok();
    let config = match starter::config::Config::from_env() {
        Ok(config) => Arc::new(config),
        Err(error) => {
            eprintln!("configuration: {error}");
            return ExitCode::from(2);
        }
    };

    let pool = starter::db::connect(&config)
        .await
        .expect("database connection");
    starter::db::migrate(&pool)
        .await
        .expect("database migrations");

    let mailer = match starter::mail::Mailer::from_config(&config) {
        Ok(mailer) => mailer,
        Err(error) => {
            eprintln!("mail: {error}");
            return ExitCode::from(2);
        }
    };
    let addr = config.bind_addr.clone();
    let app = starter::server::router(pool, config, mailer);

    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind address");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("serve");
    ExitCode::SUCCESS
}
