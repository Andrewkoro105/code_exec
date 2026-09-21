use std::{
    ffi::OsStr,
    io,
    process::{ExitStatus, Stdio},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    ExitStatus(ExitStatus, String),
    RunError(String),
}

pub enum ErrorDetector {
    StderrNotIsEmpty,
}

pub struct Runner {
    pub child: Child,
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
    pub stderr: ChildStderr,
    error_detector: ErrorDetector,
}

impl Runner {
    pub fn new(command: impl AsRef<OsStr>, error_detector: ErrorDetector) -> io::Result<Self> {
        let mut child = Command::new(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Self {
            stdin: child.stdin.take().unwrap(),
            stdout: child.stdout.take().unwrap(),
            stderr: child.stderr.take().unwrap(),
            child,
            error_detector,
        })
    }

    pub async fn run(&mut self, script: String, end_marker: String) -> Result<String, Error> {
        self.stdin
            .write_all(script.as_bytes())
            .await
            .map_err(Error::Io)?;
        self.stdin
            .write_all("\n".as_bytes())
            .await
            .map_err(Error::Io)?;
        self.stdin.flush().await.map_err(Error::Io)?;

        let mut buf_reader = BufReader::new(&mut self.stdout);
        let mut out = String::new();
        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line).await.map_err(Error::Io)?;
            out = format!("{out}\n{line}");

            let end_idx = out.rfind(&end_marker);
            if end_idx.is_some() {
                break Ok(out);
            }

            if let Some(exit_status) = self.child.try_wait().map_err(Error::Io)? {
                let mut stderr_str = String::new();
                self.stderr
                    .read_to_string(&mut stderr_str)
                    .await
                    .map_err(Error::Io)?;

                break Err(Error::ExitStatus(exit_status, stderr_str));
            }

            match self.error_detector {
                ErrorDetector::StderrNotIsEmpty => {
                    if !BufReader::new(&mut self.stderr).buffer().is_empty() {
                        let mut stderr_str = String::new();
                        self.stderr
                            .read_to_string(&mut stderr_str)
                            .await
                            .map_err(Error::Io)?;
                        if !stderr_str.is_empty() {
                            break Err(Error::RunError(stderr_str));
                        }
                    }
                }
            }
        }
    }
}
