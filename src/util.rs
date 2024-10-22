use std::future::Future;

use tracing::error;

pub async fn send_or_log<T>(send: impl Future<Output = serenity::Result<T>>) {
    if let Err(e) = send.await {
        error!(error = "Unable to send message/reply", err = e.to_string());
    }
}
