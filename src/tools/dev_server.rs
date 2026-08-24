use crate::utils::net::is_port_available;
use anyhow::{bail, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};

pub struct DevServerManager {
    working_dir: PathBuf,
    port: u16,
    child_process: Option<Child>,
}

impl DevServerManager {
    pub fn new(working_dir: PathBuf, port: u16) -> Self {
        Self {
            working_dir,
            port,
            child_process: None,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn start(&mut self) -> Result<()> {
        if !is_port_available(self.port) {
            bail!("Port {} is already occupied", self.port);
        }

        let child = Command::new("python3")
            .arg("-m")
            .arg("http.server")
            .arg(self.port.to_string())
            .current_dir(&self.working_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        self.child_process = Some(child);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child_process.take() {
            let _ = child.kill().await;
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.child_process.is_some()
    }
}

impl Drop for DevServerManager {
    fn drop(&mut self) {
        if let Some(mut child) = self.child_process.take() {
            let _ = child.start_kill();
        }
    }
}
