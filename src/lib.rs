use tower_http::cors::{Any, CorsLayer};
use tower_service::Service;
use worker::*;

mod auth;
mod backup;
mod crypto;
mod db;
mod error;
mod handlers;
mod models;
mod router;

#[event(fetch)]
pub async fn main(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    // Set up logging
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    // Allow all origins for CORS, which is typical for a public API like Bitwarden's.
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let mut app = router::api_router(env).layer(cors);

    Ok(app.call(req).await?)
}

/// 定时任务：每天自动备份到坚果云
#[event(scheduled)]
pub async fn scheduled(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    console_log!("Starting scheduled backup...");
    
    match backup::backup_to_jianguoyun(&env).await {
        Ok(msg) => console_log!("Scheduled backup completed: {}", msg),
        Err(e) => console_error!("Scheduled backup failed: {:?}", e),
    }
}
