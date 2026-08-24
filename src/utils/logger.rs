use anyhow::Result;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logger(log_file_path: Option<&Path>) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,frontharness=debug"));

    if let Some(path) = log_file_path {
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let file_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(file);
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().with_ansi(true))
            .init();
    }

    Ok(())
}

pub fn log_raw(file_path: &Path, message: &str) -> Result<()> {
    if let Some(parent) = file_path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    writeln!(file, "[{}] {}", timestamp, message)?;
    Ok(())
}
