use std::process::{Command, Stdio};
use std::thread;
use tracing::{info, warn};

pub fn boot() {
    thread::spawn(|| {
        info!("[SCOUT_SUPERVISOR] Starting scout_daemon.py in background thread.");
        loop {
            // Depending on where `engram` is run from, we should ideally resolve the absolute path.
            // For now, assuming we're in the repository root or we fallback to an absolute path if it fails.
            let daemon_path = "integrations/scout_daemon.py";
            let fallback_path = "/home/a/Documents/Engram/integrations/scout_daemon.py";

            let path_to_run = if std::path::Path::new(daemon_path).exists() {
                daemon_path
            } else {
                fallback_path
            };

            let mut child = match Command::new("python3")
                .arg(path_to_run)
                .stdout(Stdio::inherit())
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
