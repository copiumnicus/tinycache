use super::store;
use crate::store::StoreErr;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};
#[cfg(feature = "tracing")]
use tracing::{debug, warn};

/// cache with optional max age configuration
#[derive(Clone)]
pub struct TinyRef {
    cache_name: String,
    max_cache_age: Option<Duration>,
    ignore_cache: bool,
}

const CACHE_NAME: &str = ".tiny_cache";

impl TinyRef {
    pub fn with_name(cache_name: impl ToString) -> Self {
        Self {
            cache_name: cache_name.to_string(),
            max_cache_age: None,
            ignore_cache: false,
        }
    }
    pub fn new() -> Self {
        Self {
            cache_name: CACHE_NAME.to_string(),
            max_cache_age: None,
            ignore_cache: false,
        }
    }
    /// define max age of the cache
    pub fn max_age(mut self, max_duration: Duration) -> Self {
        self.max_cache_age = Some(max_duration);
        self
    }

    pub fn no_cache(&self) -> Self {
        let mut inner = self.clone();
        inner.ignore_cache = true;
        inner
    }
}

impl TinyRef {
    /// get cached item from cache or use fun to resolve the item
    pub fn get_cached_or_fetch<T, Fun>(&self, item_key: impl ToString, fetch_with: Fun) -> T
    where
        for<'a> T: Deserialize<'a> + Serialize + Debug,
        Fun: FnOnce() -> T,
    {
        let k = item_key.to_string();
        if !self.ignore_cache {
            if let Some(res) = self.read(k.clone()) {
                return res;
            }
        } else {
            #[cfg(feature = "tracing")]
            debug!("Ignoring cache on {:?}", k);
        }
        let item_from_fut = fetch_with();
        self.write(item_key, &item_from_fut);
        item_from_fut
    }
    pub fn write<T>(&self, item_key: impl ToString, v: &T)
    where
        for<'a> T: Deserialize<'a> + Serialize + Debug,
    {
        let item_key = item_key.to_string();
        if let Err(e) = store::write(self.cache_name.clone(), item_key.clone(), v) {
            #[cfg(feature = "tracing")]
            warn!(
                "failed to write to cache `{}` -> `{}` {:?}",
                item_key, self.cache_name, e
            );
        } else {
            #[cfg(feature = "tracing")]
            debug!("SAVE `{}` -> `{}`", item_key, self.cache_name);
        }
    }
    pub fn item_age(&self, item_key: impl ToString) -> Option<Duration> {
        store::item_age(self.cache_name.clone(), item_key.to_string()).ok()
    }
    pub fn read<T>(&self, item_key: impl ToString) -> Option<T>
    where
        for<'a> T: Deserialize<'a> + Serialize + Debug,
    {
        let item_key = item_key.to_string();
        // if max age for the cache is defined
        if let Some(max_age) = self.max_cache_age {
            if let Some(age) = self.item_age(item_key.clone()) {
                if age > max_age {
                    #[cfg(feature = "tracing")]
                    debug!("CACHE `{}` -> TOO OLD. AGE: {:?}", item_key, age);
                    self.invalidate(item_key);
                    return None;
                }
            }
        }
        match store::read::<T>(self.cache_name.clone(), item_key.clone()) {
            Ok(v) => Some(v),
            Err(e) => {
                if let StoreErr::Ser(e) = e {
                    // invalidate the cache
                    #[cfg(feature = "tracing")]
                    warn!(
                        "Failed to deserialize {:?}, invalidating cache -> `{}`",
                        e, item_key
                    );
                    self.invalidate(item_key);
                }
                None
            }
        }
    }
    pub fn invalidate(&self, item_key: impl ToString) {
        if let Err(e) = store::remove(self.cache_name.clone(), item_key.to_string()) {
            #[cfg(feature = "tracing")]
            warn!("Failed to invalidate cache {:?}", e);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cache() {
        let tiny = TinyRef::new();
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        pub struct TestStruct {
            a: String,
        }
        let test_struct = TestStruct { a: "hello".into() };
        let key = "testval";

        // write read
        tiny.write(key, &test_struct);
        let stored_struct: TestStruct = tiny.read(key).unwrap();
        assert_eq!(stored_struct, test_struct);

        // test item age
        std::thread::sleep(Duration::from_millis(100));
        let age = tiny.item_age(key).unwrap();
        println!("age {:?}", age);
        assert!(age > Duration::from_millis(50));

        // invalidate
        tiny.invalidate(key);
        assert_eq!(tiny.read::<TestStruct>(key), None);
    }
}
