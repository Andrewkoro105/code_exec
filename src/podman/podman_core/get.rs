use crate::podman::podman_core::PodmanCore;
use async_compression::tokio::bufread::GzipDecoder;
use futures_util::StreamExt;
use std::{env, path::Path};
use sugar::hashmap;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_tar::Archive;
use tokio_util::io::StreamReader;
use tracing::debug;

pub enum Version {
    Last,
    Custom(String),
}

impl PodmanCore {
    async fn get_ver(version: &Version) -> Result<String, Box<dyn std::error::Error>> {
        Ok(match version {
            Version::Last => {
                let client = reqwest::Client::new();

                let json: serde_json::Value = client
                    .get("https://api.github.com/repos/mgoltzsche/podman-static/releases/latest")
                    .header("User-Agent", "my-app")
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;

                json["tag_name"]
                    .as_str()
                    .ok_or("no tag_name in response")?
                    .into()
            }
            Version::Custom(ver) => ver.clone(),
        })
    }

    pub async fn get(dir: &Path, version: Version) -> Result<Self, Box<dyn std::error::Error>> {
        let version = Self::get_ver(&version).await?;
        let dir = &dir.join(&version);
        let podman_local_path = dir.join("podman-linux-amd64").join("usr").join("local");
        let env_podman_path = podman_local_path.join("bin");
        let path = env_podman_path.join("podman");
        let conf_path = dir.join("containers.conf");
        let policy_path = dir.join("policy.json");
        if !dir.exists() {
            fs::create_dir_all(dir).await?;

            let url = format!(
                "https://github.com/mgoltzsche/podman-static/releases/download/{version}/podman-linux-amd64.tar.gz",
            );
            debug!("download {url}");

            let response = reqwest::get(url).await?;
            let stream = response.bytes_stream().map(|result| {
                result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            });
            let reader = StreamReader::new(stream);
            let buf_reader = tokio::io::BufReader::new(reader);
            let decompressor = GzipDecoder::new(buf_reader);
            let mut archive = Archive::new(decompressor);
            archive.unpack(dir).await?;

            let conmon_path =
                fs::canonicalize(podman_local_path.join("lib").join("podman").join("conmon"))
                    .await?;
            let crun_path = fs::canonicalize(env_podman_path.join("crun")).await?;
            let helper_binaries_dir =
                fs::canonicalize(podman_local_path.join("lib").join("podman")).await?;

            File::options()
                .write(true)
                .create(true)
                .open(&conf_path)
                .await?
                .write_all(
                    format!(
                        r#"
[engine]
conmon_path = [{conmon_path:?}]
helper_binaries_dir = [{helper_binaries_dir:?}]

[engine.runtimes]
crun = [{crun_path:?}]
"#
                    )
                    .as_bytes(),
                )
                .await?;
            File::options()
                .write(true)
                .create(true)
                .open(&policy_path)
                .await?
                .write_all(
                    r#"
{
  "default": [
    {
      "type": "insecureAcceptAnything"
    }
  ],
  "transports": {
    "docker": {},
    "docker-daemon": {
      "": [
        {
          "type": "insecureAcceptAnything"
        }
      ]
    }
  }
}
"#
                    .as_bytes(),
                )
                .await?;
        }

        Ok(PodmanCore::new(
            path,
            version,
            hashmap! {
                "PATH".into() => format!("{}:{}", env::var("PATH").unwrap(), fs::canonicalize(env_podman_path).await?.to_string_lossy().to_string()),
                "CONTAINERS_CONF".into() => fs::canonicalize(conf_path).await?.to_string_lossy().into(),
                "CONTAINERS_POLICY_JSON".into() => fs::canonicalize(policy_path).await?.to_string_lossy().into(),
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use tokio::fs;

    use crate::podman::podman_core::{PodmanCore, Version};
    use std::path::Path;

    // #[tokio::test]
    // async fn get_podman() {
    //     let ver = "v5.8.7";
    //     let podman = PodmanCore::get(&Path::new("./tmp/test/"), Version::Custom(ver.into()))
    //         .await
    //         .unwrap();
    //     assert_eq!(
    //         podman,
    //         PodmanCore::new(
    //             format!("./tmp/test/{ver}/podman-linux-amd64/usr/local/bin/podman").into(),
    //             ver.into(),
    //             hashmap! {
    //                 "PATH".into() => env_podman_path.to_string_lossy().into()
    //             }
    //         )
    //     );
    //     assert!(podman.path.exists());
    //     fs::remove_dir_all("./tmp").await.unwrap();
    // }

    #[tokio::test]
    async fn get_last_podman() {
        let podman = PodmanCore::get(&Path::new("./tmp/test/"), Version::Last)
            .await
            .unwrap();
        assert!(podman.path.exists());
        fs::remove_dir_all("./tmp").await.unwrap();
    }
}
