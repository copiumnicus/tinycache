use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{fs, path::PathBuf, time::Duration};

// need:
// - remove
// - metadata
// - write
// - read

#[derive(Debug)]
pub(crate) enum StoreErr {
    /// serialization level err
    Ser(bincode::Error),
    /// io level err
    IO(std::io::Error),
    /// when can't get age of the file
    Time(std::time::SystemTimeError),
}

impl StoreErr {
    fn ser(e: bincode::Error) -> Self {
        Self::Ser(e)
    }
    fn io(e: std::io::Error) -> Self {
        Self::IO(e)
    }
    fn time(e: std::time::SystemTimeError) -> Self {
        Self::Time(e)
    }
}

/// creates if does not exist
/// corresponds to mkdir so if already exists it fails
/// do not check or return result because cache has to be simple
#[allow(unused_must_use)]
fn create_cache(name: &String) {
    fs::create_dir(name);
}

fn path_join(cache_name: String, key: String) -> PathBuf {
    std::path::Path::new(&cache_name).join(key)
}

fn fmt_key(k: String) -> String {
    let mut hasher = Sha1::new();
    hasher.update(k.as_bytes());
    let result: Vec<u8> = hasher.finalize().to_vec();
    format!(
        "{}",
        result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join("")
    )
}

pub(crate) fn read<T>(cache_name: String, key: String) -> Result<T, StoreErr>
where
    T: for<'a> Deserialize<'a>,
{
    let key = fmt_key(key);
    let path = path_join(cache_name, key);
    let bytes = fs::read(path).map_err(StoreErr::io)?;
    Ok(bincode::deserialize(&bytes).map_err(StoreErr::ser)?)
}

pub(crate) fn write<T>(cache_name: String, key: String, value: &T) -> Result<(), StoreErr>
where
    T: Serialize,
{
    let key = fmt_key(key);
    create_cache(&cache_name);
    let path = path_join(cache_name, key);
    let bytes = bincode::serialize(value).map_err(StoreErr::ser)?;
    fs::write(path, bytes).map_err(StoreErr::io)?;
    Ok(())
}

pub(crate) fn remove(cache_name: String, key: String) -> Result<(), StoreErr> {
    let key = fmt_key(key);
    let path = path_join(cache_name, key);
    fs::remove_file(path).map_err(StoreErr::io)?;
    Ok(())
}

/// returns the age of the cache item
pub(crate) fn item_age(cache_name: String, key: String) -> Result<Duration, StoreErr> {
    let key = fmt_key(key);
    let path = path_join(cache_name, key);
    let metadata = fs::metadata(path).map_err(StoreErr::io)?;
    let sys_time = metadata.modified().map_err(StoreErr::io)?;
    let age = sys_time.elapsed().map_err(StoreErr::time)?;
    Ok(age)
}
