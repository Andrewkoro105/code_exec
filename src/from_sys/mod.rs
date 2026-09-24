pub mod matlab_like;
pub mod runner;

use crate::{
    clean::Clean,
    from_sys::runner::{ErrorDetector, Runner},
    run::Run,
    run_script::RunScript,
    set_init_script::SetInitScript,
    values::Values,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub enum FromSysError {
    Io(std::io::Error),
    Runner(runner::Error),
    IncorrectScriptOutput {
        out: String,
        start_marker: String,
        end_marker: String,
    },
}

pub struct FromSys {
    pub base_command: String,
    pub print_value_pattern: String,
    pub input_value_pattern: String,

    runner: Option<Runner>,
    init_script: Option<String>,
}

impl RunScript for FromSys {
    type Script = String;
    type Error = FromSysError;

    async fn run_script(
        &self,
        script: Self::Script,
        data: HashMap<String, JsonValue>,
    ) -> Result<Values, Self::Error> {
        let script: String = self.get_script(script, &data)?;
        debug!("{script}");
        self.get_data({
            let mut runner = Runner::new(&self.base_command, ErrorDetector::StderrNotIsEmpty)
                .map_err(Self::Error::Io)?;
            if let Some(init_script) = self.init_script.clone() {
                runner
                    .run(init_script, Self::get_end_out_block().replace("\\n", "\n"))
                    .await
                    .map_err(Self::Error::Runner)?;
            }
            runner
                .run(script, Self::get_end_out_block().replace("\\n", "\n"))
                .await
                .map_err(Self::Error::Runner)?
        })
    }
}

impl Run for FromSys {
    type Script = String;

    type Error = FromSysError;

    async fn run(
        &mut self,
        script: Self::Script,
        data: HashMap<String, JsonValue>,
    ) -> Result<Values, Self::Error> {
        let script = self.get_script(script, &data)?;
        debug!("{script}");

        if self.runner.is_none() {
            self.runner = Some(
                Runner::new(&self.base_command, ErrorDetector::StderrNotIsEmpty)
                    .map_err(Self::Error::Io)?,
            );
            if let Some(init_script) = self.init_script.clone() {
                self.runner
                    .as_mut()
                    .unwrap()
                    .run(init_script, Self::get_end_out_block().replace("\\n", "\n"))
                    .await
                    .map_err(Self::Error::Runner)?;
            }
        }

        let out = self
            .runner
            .as_mut()
            .unwrap()
            .run(script, Self::get_end_out_block().replace("\\n", "\n"))
            .await
            .map_err(Self::Error::Runner)?;
        self.get_data(out)
    }
}

impl SetInitScript for FromSys {
    type Script = String;

    type Error = FromSysError;

    fn set_init_script(
        &mut self,
        script: Self::Script,
        data: HashMap<String, JsonValue>,
    ) -> Result<(), Self::Error> {
        self.init_script = Some(self.get_script(script, &data)?);
        Ok(())
    }
}

impl Clean for FromSys {
    type Script = String;

    type Error = FromSysError;

    async fn clean(&mut self) -> Result<(), Self::Error> {
        if let Some(runner) = self.runner.as_mut() {
            runner.child.kill().await.map_err(FromSysError::Io)?;
            runner.child.wait().await.map_err(FromSysError::Io)?;

            self.runner = None;
        }
        Ok(())
    }
}

impl FromSys {
    pub fn clone_conf(&self) -> Self {
        Self {
            base_command: self.base_command.clone(),
            print_value_pattern: self.print_value_pattern.clone(),
            input_value_pattern: self.input_value_pattern.clone(),
            runner: None,
            init_script: self.init_script.clone(),
        }
    }

    pub fn set_runner(&mut self, runner: Runner) {
        self.runner = Some(runner)
    }

    pub fn get_runner(&self) -> &Option<Runner> {
        &self.runner
    }

    fn get_script(
        &self,
        script: String,
        data: &HashMap<String, JsonValue>,
    ) -> Result<String, FromSysError> {
        let mut base_script = script.trim().to_string();
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
            if data.is_empty() {
                "".into()
            } else {
                self.input_value_pattern.replace(
                    "{}",
                    &format!(
                        "\"{}\"",
                        serde_json::to_string(&data).unwrap().replace("\"", "\\\"")
                    ),
                )
            },
            self.print_value_pattern,
        );
        Ok(base_script)
    }

    fn get_data(&self, out: String) -> Result<Values, FromSysError> {
        let start_marker = Self::get_start_out_block().replace("\\n", "\n");
        let end_marker = Self::get_end_out_block().replace("\\n", "\n");

        if let Some(start_idx) = out.rfind(&start_marker)
            && let Some(end_idx) = out.rfind(&end_marker)
        {
            let slice_start = start_idx + start_marker.len();
            let slice_end = end_idx;

            if slice_start <= slice_end
                && out.is_char_boundary(slice_start)
                && out.is_char_boundary(slice_end)
            {
                Ok(Values::new(
                    serde_json::from_slice(out[slice_start..slice_end].as_bytes()).unwrap(),
                    self.get_result_name(),
                ))
            } else {
                Err(FromSysError::IncorrectScriptOutput {
                    out,
                    start_marker,
                    end_marker,
                })
            }
        } else {
            Err(FromSysError::IncorrectScriptOutput {
                out,
                start_marker,
                end_marker,
            })
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
