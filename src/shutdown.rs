use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Debug)]
pub enum ShutdownError {
    SignalBindFail,
}

pub fn start_shutdown_listener() -> Result<Arc<Notify>, ShutdownError> {
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    tokio::spawn(async move {
        if let Err(_) = tokio::signal::ctrl_c().await {
            return;
        }
        println!("Ctrl+C received");
        notify_clone.notify_waiters();
    });
    Ok(notify)
}
