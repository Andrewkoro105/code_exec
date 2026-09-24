use std::{ffi::OsStr, path::Path, process::Stdio};

use tokio::{fs, io::AsyncWriteExt, process::Command};
use tracing::{debug, info};

use crate::{
    from_sys::runner::{ErrorDetector, Runner},
    podman::podman_core::PodmanCore,
};

impl PodmanCore {
    pub async fn run(
        &self,
        dir: &Path,
        dockerfile: String,
        name: String,
        command: String,
    ) -> std::io::Result<Runner> {
        info!("build {name}");
        let tmpdir_env = dir.join("cache");
        if !tmpdir_env.exists() {
            fs::create_dir_all(&tmpdir_env).await?;
        }
        let tmpdir_env = fs::canonicalize(&tmpdir_env).await?;
        let mut build = Command::new(&self.path)
            .args(&[
                OsStr::new("--root"),
                dir.join("root").as_os_str(),
                OsStr::new("--runroot"),
                dir.join("runroot").as_os_str(),
                OsStr::new("--tmpdir"),
                dir.join("tmpdir").as_os_str(),
                OsStr::new("build"),
                OsStr::new("-t"),
                OsStr::new(&name),
                OsStr::new("-f"),
                OsStr::new("-"),
                OsStr::new("."),
            ])
            .env("TMPDIR", &tmpdir_env)
            .envs(&self.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        debug!(
            "{}",
            self.env
                .iter()
                .map(|(k, v)| format!("export {k}={v}"))
                .collect::<Vec<_>>()
                .join("\n")
        );

        debug!("build input");

        let mut build_stdin = build.stdin.take().unwrap();
        build_stdin.write_all(dockerfile.as_bytes()).await?;
        build_stdin.flush().await?;
        drop(build_stdin);
        debug!("build end input");
        let output = build.wait_with_output().await?;
        debug!("build end");

        if !output.status.success() {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("out: {}; err: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)),
            ))?;
        }

        info!("run {name}");
        Runner::from_command(
            Command::new(&self.path)
                .args(&[
                    OsStr::new("--root"),
                    dir.join("root").as_os_str(),
                    OsStr::new("--runroot"),
                    dir.join("runroot").as_os_str(),
                    OsStr::new("--tmpdir"),
                    dir.join("tmpdir").as_os_str(),
                    OsStr::new("run"),
                    OsStr::new("--rm"),
                    OsStr::new("-i"),
                    OsStr::new(&name),
                    OsStr::new(&command),
                ])
                .env("TMPDIR", dir.join("cache"))
                .envs(&self.env),
            ErrorDetector::StderrNotIsEmpty,
        )
    }
}
