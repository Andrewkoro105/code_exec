use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::{File, OpenOptions},
    io,
    path::Path,
};

pub trait Install<'de>
where
    Self: Sized,
{
    type Error: From<io::Error> + From<serde_json::Error>;
    type Target;
    type LockData: Serialize + DeserializeOwned;

    fn create(self) -> Result<(Self::Target, Self::LockData), Self::Error>;

    fn install(lock_data: Self::LockData) -> Result<Self::Target, Self::Error>;

    fn load(self, path: &Path) -> Result<Self::Target, Self::Error> {
        if path.exists() {
            Self::install(
                serde_json::from_reader(File::open(path).map_err(Into::<Self::Error>::into)?)
                    .map_err(Into::<Self::Error>::into)?,
            )
        } else {
            match self.create() {
                Ok((result, lock_data)) => {
                    serde_json::to_writer_pretty(
                        OpenOptions::new()
                            .create(true)
                            .write(true)
                            .open(path)
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

    fn load_host(path: &Path) -> Result<Self::Target, Self::Error> where Self: Host{
        Self::get_host().load(path)
    }
}

pub trait Host {
    fn get_host() -> Self;
}
