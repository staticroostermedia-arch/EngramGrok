use std::process::{Command, Stdio};
use std::thread;
use tracing::{info, warn};

pub fn boot() {
    thread::spawn(|| {
        info!("[SCOUT_SUPERVISOR] Starting scout_daemon.py in background thread.");
        loop {
            // Prefer a relative path from the binary; allow override via ENGRAM_SCOUT_DAEMON env var.
            let daemon_path = "integrations/scout_daemon.py";
            let env_path = std::env::var("ENGRAM_SCOUT_DAEMON").unwrap_or_default();
            let path_to_run: &str = if !env_path.is_empty() && std::path::Path::new(&env_path).exists() {
                &env_path
            } else if std::path::Path::new(daemon_path).exists() {
                daemon_path
            } else {
                warn!("[SCOUT_SUPERVISOR] scout_daemon.py not found. Set ENGRAM_SCOUT_DAEMON to override path.");
                thread::sleep(std::time::Duration::from_secs(30));
                continue;
            };

            let mut child = match Command::new("python3")
                .arg(path_to_run)
                .stdout(Stdio::null())
                .stderr(Stdio::inherit())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    warn!("[SCOUT_SUPERVISOR] Failed to spawn scout daemon: {}. Retrying in 10s...", e);
                    thread::sleep(std::time::Duration::from_secs(10));
                    continue;
                }
            };

            match child.wait() {
                Ok(status) => {
                    warn!("[SCOUT_SUPERVISOR] scout_daemon.py exited with {}. Restarting in 5s...", status);
                }
                Err(e) => {
                    warn!("[SCOUT_SUPERVISOR] Failed to wait on scout_daemon.py: {}. Restarting in 5s...", e);
                }
            }

            thread::sleep(std::time::Duration::from_secs(5));
        }
    });
}
