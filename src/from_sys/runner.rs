use std::{
    ffi::OsStr,
    io::{self, BufRead, BufReader, Write},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
};

pub struct Runner {
    pub child: Child,
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
    pub stderr: ChildStderr,
}

impl Runner {
    pub fn new(command: impl AsRef<OsStr>) -> io::Result<Self> {
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
        })
    }

    pub fn run(&mut self, script: String, end_marker: String) -> io::Result<String> {
        self.stdin.write_all(script.as_bytes())?;
        self.stdin.write_all("\n".as_bytes())?;

        self.stdin.flush()?;

        let mut buf_reader = BufReader::new(&mut self.stdout);
        let mut out = String::new();
        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line)?;
            out = format!("{out}\n{line}");

            let end_idx = out.rfind(&end_marker);

            if end_idx.is_some() {
                break Ok(out);
            }
        }
    }
}
