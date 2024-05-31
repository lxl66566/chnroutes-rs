use std::{env, env::temp_dir, io, path::PathBuf, time::Duration};

use crate::error::CacheError;

/// A file Cache with an expire time, to store IP source file for a few (7)
/// days.
pub struct Cache {
    name: String,
    expire_time: Duration,
}

impl Cache {
    pub fn new(name: impl AsRef<str>, expire_time: Duration) -> Self {
        Self {
            name: name.as_ref().to_string(),
            expire_time,
        }
    }

    /// Get the path of the cache file
    fn get_path(&self) -> PathBuf {
        temp_dir().join(env!("CARGO_PKG_NAME")).join(&self.name)
    }

    /// Save file to cache
    pub fn save<'a>(&self, bytes: &'a [u8]) -> io::Result<&'a [u8]> {
        let path = self.get_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, bytes)?;
        Ok(bytes)
    }

    /// Load file from cache, if the file not found or expired, return [`None`]
    pub fn load(&self) -> std::result::Result<Option<Vec<u8>>, CacheError> {
        let path = self.get_path();
        if path.exists() {
            let metadata = std::fs::metadata(&path)?;
            let last_modified = metadata.modified()?;
            if last_modified.elapsed().unwrap_or(self.expire_time) < self.expire_time {
                return Ok(Some(std::fs::read(path)?));
            }
        }
        Ok(None)
    }

    pub fn save_str<'a>(&self, bytes: &'a str) -> std::result::Result<&'a str, CacheError> {
        self.save(bytes.as_bytes())?;
        Ok(bytes)
    }

    #[allow(unused)]
    pub fn remove(&self) -> std::result::Result<(), CacheError> {
        let path = self.get_path();
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache() {
        let cache = Cache::new("test", Duration::from_millis(20));
        let path = cache.get_path();
        assert!(!path.exists());
        cache.save_str("test").unwrap();
        assert!(path.exists());
        assert_eq!(cache.load().unwrap().unwrap(), "test".as_bytes());
        std::thread::sleep(Duration::from_millis(25));
        assert!(cache.load().unwrap().is_none());
        cache.remove().unwrap();
    }
}
