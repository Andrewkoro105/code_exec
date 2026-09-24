mod podman_core;

use serde::{Deserialize, Serialize};

use crate::{
    from_sys::{FromSys, FromSysError},
    load::Load,
    podman::podman_core::{PodmanCore, Version},
    run::Run,
    run_script::RunScript,
};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    PodmanCore(Box<dyn std::error::Error>),
    FromSys(FromSysError)
}

pub struct PodmanBuilder {
    from_sys: FromSys,
    docker_file: String,
    dir: PathBuf,
    name: String,
}

pub struct Podman {
    from_sys: FromSys,
    podman_core: PodmanCore,
    docker_file: String,
    dir: PathBuf,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodmanLockData {
    podman_version: String,
}

impl PodmanLockData {
    pub fn new(podman_version: String) -> Self {
        PodmanLockData { podman_version }
    }

    pub fn get_podman_version(&self) -> &String {
        &self.podman_version
    }
}

impl PodmanBuilder {
    pub fn new(docker_file: String, dir: PathBuf, from_sys: FromSys, name: String) -> Self {
        Self {
            from_sys,
            docker_file,
            dir,
            name,
        }
    }
}

impl Load for PodmanBuilder {
    type LockData = PodmanLockData;

    type Target = Podman;

    type Error = Error;

    async fn create(&self) -> Result<(Self::Target, Self::LockData), Self::Error> {
        let podman_core = PodmanCore::get(&self.dir, Version::Last)
            .await
            .map_err(Self::Error::PodmanCore)?;
        let version = podman_core.get_version().clone();
        Ok((
            Self::Target {
                from_sys: self.from_sys.clone_conf(),
                podman_core: podman_core,
                docker_file: self.docker_file.clone(),
                dir: self.dir.clone(),
                name: self.name.clone(),
            },
            PodmanLockData::new(version),
        ))
    }

    async fn from_lock_data(&self, lock_data: Self::LockData) -> Result<Self::Target, Self::Error> {
        let podman_core = PodmanCore::get(&self.dir, Version::Custom(lock_data.podman_version))
            .await
            .map_err(Self::Error::PodmanCore)?;
        Ok(Self::Target {
            from_sys: self.from_sys.clone_conf(),
            podman_core: podman_core,
            docker_file: self.docker_file.clone(),
            dir: self.dir.clone(),
            name: self.name.clone(),
        })
    }
}

impl Run for Podman {
    type Script = <FromSys as RunScript>::Script;

    type Error = Error;

    async fn run(
        &mut self,
        script: Self::Script,
        data: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<crate::values::Values, Self::Error> {
        if self.from_sys.get_runner().is_none() {
            self.from_sys.set_runner(
                self.podman_core
                    .run(
                        &self.dir,
                        self.docker_file.clone(),
                        self.name.clone(),
                        self.from_sys.base_command.clone(),
                    )
                    .await
                    .map_err(Self::Error::Io)?,
            );
        }

        self.from_sys.run(script, data).await.map_err(Self::Error::FromSys)
    }
}

impl RunScript for Podman {
    type Script = <FromSys as RunScript>::Script;

    type Error = Error;

    async fn run_script(
        &self,
        script: Self::Script,
        data: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<crate::values::Values, Self::Error> {
        let mut from_sys = self.from_sys.clone_conf();
        if from_sys.get_runner().is_none() {
            from_sys.set_runner(
                self.podman_core
                    .run(
                        &self.dir,
                        self.docker_file.clone(),
                        self.name.clone(),
                        self.from_sys.base_command.clone(),
                    )
                    .await
                    .map_err(Self::Error::Io)?,
            );
        }

        from_sys.run(script, data).await.map_err(Self::Error::FromSys)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        from_sys::matlab_like::MatLabLikeBuilder,
        load::Load,
        podman::{Error, PodmanBuilder},
        run::Run,
    };
    use serde_json::{Number, Value as JsonValue};
    use std::collections::HashMap;

    #[tokio::test]
    async fn run() -> Result<(), Error> {
        let (mut octave, _) = PodmanBuilder::new(
            "FROM gnuoctave/octave:9.2.0".into(),
            "./tmp/podman/".into(),
            MatLabLikeBuilder {
                target: "octave".into(),
            }
            .build(),
            "octave".into(),
        )
        .create()
        .await?;

        let mut data = HashMap::new();
        data.insert(
            "test_value".to_string(),
            JsonValue::Number(Number::from_u128(42).unwrap()),
        );

        octave
            .run(
                "input_data.test_value = input_data.test_value * 2"
                    .to_string()
                    .into(),
                data,
            )
            .await?;
        let result = octave
            .run(
                "input_data.test_value + 2".to_string().into(),
                HashMap::new(),
            )
            .await?
            .get_result()
            .as_u64()
            .unwrap();
        assert_eq!(result, 42 * 2 + 2);

        Ok(())
    }
}
