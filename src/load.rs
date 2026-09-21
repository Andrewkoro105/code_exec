//! A trait that ensures the standardization of the process for creating a reproducible system.
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::{File, OpenOptions},
    io,
    path::Path,
};

/// A trait that ensures the standardization of the process for creating a reproducible system.
pub trait Load
where
    Self: Sized,
{
    /// Data that is guaranteed to fully describe the system so that it can be fully reproduced on supported devices.
    type LockData: Serialize + DeserializeOwned;
    /// Type of System Being Developed
    type Target;

    type Error: From<io::Error> + From<serde_json::Error>;

    /// It provides access to the latest version of the system for this platform and returns data for subsequent system playback on any supported device.
    fn create(&self) -> impl Future<Output = Result<(Self::Target, Self::LockData), Self::Error>>;

    /// Provides access to the system described.
    fn from_lock_data(
        &self,
        lock_data: Self::LockData,
    ) -> impl Future<Output = Result<Self::Target, Self::Error>>;

    /// Loads the system based on the `lock_data` file at the specified path. If no data is found at the specified path, the system will be created using the provided function, and its `lock_data` will be saved at the specified path.
    fn load_core<'a, FR>(
        &'a self,
        lock_data_path: &Path,
        create: impl FnOnce(&'a Self) -> FR,
    ) -> impl Future<Output = Result<Self::Target, Self::Error>>
    where
        FR: Future<Output = Result<(Self::Target, Self::LockData), Self::Error>> + 'a,
    {
        async move {
            if lock_data_path.exists() {
                self.from_lock_data(
                    serde_json::from_reader(
                        File::open(lock_data_path).map_err(Into::<Self::Error>::into)?,
                    )
                    .map_err(Into::<Self::Error>::into)?,
                )
                .await
            } else {
                match create(self).await {
                    Ok((result, lock_data)) => {
                        serde_json::to_writer_pretty(
                            OpenOptions::new()
                                .create(true)
                                .write(true)
                                .open(lock_data_path)
                                .map_err(Into::<Self::Error>::into)?,
                            &lock_data,
                        )
                        .map_err(Into::<Self::Error>::into)?;
                        Ok(result)
                    }
                    Err(err) => Err(err.into()),
                }
            }
        }
    }

    /// Loads the system based on the `lock_data` file located at the specified path. If no data is found at the specified path, the system will be created using [`create()`](Self::create), and its `lock_data` file will be saved at the specified path.
    fn load(
        &self,
        lock_data_path: &Path,
    ) -> impl Future<Output = Result<Self::Target, Self::Error>> {
        self.load_core(lock_data_path, Self::create)
    }

    /// Loads the system based on the `lock_data` file located at the specified path. If no data is found at the specified path, the system will be created using [`get_host()`](Host::get_host), and its `lock_data` file will be saved at the specified path.
    fn load_host(
        &self,
        lock_data_path: &Path,
    ) -> impl Future<Output = Result<<Self as Load>::Target, <Self as Load>::Error>>
    where
        Self: Host<
                LockData = <Self as Load>::LockData,
                Target = <Self as Load>::Target,
                Error = <Self as Load>::Error,
            >,
    {
        self.load_core(lock_data_path, Self::get_host)
    }
}

/// Provides access, where possible, to the installed version of the system or to the latest version of the system for that platform. It also returns data for subsequent system playback on any supported device.
pub trait Host {
    type Error: From<io::Error> + From<serde_json::Error>;
    type Target;
    type LockData: Serialize + DeserializeOwned;

    /// Provides access, where possible, to the installed version of the system or to the latest version of the system for that platform. It also returns data for subsequent system playback on any supported device.
    fn get_host(&self)
    -> impl Future<Output = Result<(Self::Target, Self::LockData), Self::Error>>;
}
