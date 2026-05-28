pub mod script;
pub mod matlab_like;

use script::{Script, ScriptInspector};
use serde_json::Value as JsonValue;
use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader, Write},
    process::{ChildStdout, Command, Stdio},
};
use tracing::debug;

use crate::{from_sys::script::{InspectorError, MatParser}, run_script::RunScript, values::Values};

#[derive(Debug)]
pub enum FromSysError {
    Inspector(InspectorError),
    Io(std::io::Error),
}

pub struct FromSys {
    pub base_command: String,
    pub print_value_pattern: String,
    pub input_value_pattern: String,
    pub script_inspector: ScriptInspector,
}

impl RunScript for FromSys {
    type Script = Script;
    type Error = FromSysError;

    fn run_script(
        &self,
        script: Self::Script,
        data: HashMap<String, JsonValue>,
    ) -> Result<Values, Self::Error> {
        let script: String = self.get_script(script, &data)?;
        debug!("{script}");

        let mut child = Command::new(&self.base_command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(FromSysError::Io)?;

        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();

        stdin
            .write_all(script.as_bytes())
            .map_err(FromSysError::Io)?;
        stdin.write_all("\n".as_bytes()).map_err(FromSysError::Io)?;

        stdin.flush().map_err(FromSysError::Io)?;

        self.get_data(&mut stdout)
    }
}

impl FromSys {

    fn get_script(
        &self,
        script: Script,
        data: &HashMap<String, JsonValue>,
    ) -> Result<String, FromSysError> {
        let mut base_script = self
            .script_inspector
            .to_string(script)
            .map_err(FromSysError::Inspector)?
            .trim()
            .to_string();
        if !base_script.is_empty() {
            let pos = base_script.rfind('\n').map_or(0, |p| p + 1);
            base_script.insert_str(pos, &format!("{} = ", self.get_result_name()));
        }
        base_script = format!(
            r"
{}
{base_script}
{}
",
            self.input_value_pattern.replace(
                "{}",
                &format!("\"{}\"", serde_json::to_string(&data).unwrap().replace("\"", "\\\""))
            ),
            self.print_value_pattern,
        );
        Ok(base_script)
    }

    fn get_data(&self, stdout: &mut ChildStdout) -> Result<Values, FromSysError> {
        let start_marker = Self::get_start_out_block().replace("\\n", "\n");
        let end_marker = Self::get_end_out_block().replace("\\n", "\n");

        let mut buf_reader = BufReader::new(stdout);
        let mut out = String::new();
        loop {
            let mut line = String::new();
            buf_reader.read_line(&mut line).map_err(FromSysError::Io)?;
            out = format!("{out}\n{line}");

            let start_idx = out.rfind(&start_marker);
            if let Some(start_idx) = start_idx {
                let end_idx = out.rfind(&end_marker);

                if let Some(end_idx) = end_idx {
                    let slice_start = start_idx + start_marker.len();
                    let slice_end = end_idx;

                    if slice_start <= slice_end
                        && out.is_char_boundary(slice_start)
                        && out.is_char_boundary(slice_end)
                    {
                        break Ok(Values::new(
                            serde_json::from_slice(out[slice_start..slice_end].as_bytes()).unwrap(),
                            self.get_result_name(),
                        ));
                    }
                }
            }
        }
    }

    fn get_start_out_block() -> String {
        let project_name = env!("CARGO_PKG_NAME");
        format!("[{project_name} output bloc] start\\n")
    }

    fn get_end_out_block() -> String {
        let project_name = env!("CARGO_PKG_NAME");
        format!("[{project_name} output bloc] end\\n")
    }

    fn get_result_name(&self) -> String {
        format!("{}_{}_result", env!("CARGO_PKG_NAME"), self.base_command)
    }
}