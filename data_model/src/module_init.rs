use std::sync::atomic::{AtomicBool, Ordering};
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::prelude::*;
use tracing_web::{MakeConsoleWriter, performance_layer};

static IS_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// One-time setup for a dynamically loaded page module. Each module is its
/// own wasm instance with its own statics, so it must establish console
/// tracing, the authenticated user, and the fundraising config before
/// rendering. Subsequent mounts of the same instance are no-ops.
pub async fn init_page_module() -> Result<(), String> {
    if IS_INITIALIZED.load(Ordering::Relaxed) {
        return Ok(());
    }

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Must be false for browser console
        .with_timer(())
        .with_writer(MakeConsoleWriter); // Redirects to console.log
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    // The shell authenticated before loading any module; this resolves from
    // the session without a login redirect and fills this instance's statics.
    crate::get_active_user_async()
        .await
        .map_err(|err| format!("Failed to get user info: {err:#?}"))?;
    crate::load_config().await;

    IS_INITIALIZED.store(true, Ordering::Relaxed);
    Ok(())
}
