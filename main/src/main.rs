fn main() {
    commons::start_tracing();
    tracing::info!("Hello, world!");
    tracing::error!("This is an error message");
    tracing::warn!("This is a warning message");
    tracing::debug!("This is a debug message");
}
