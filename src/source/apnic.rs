use std::{io::Cursor, net::IpAddr, str::FromStr, time::Duration};

use ipnet::IpNet;
use log::{info, warn};

use crate::cache::Cache;

/// Fetch IP data from apnic.net, add it to cache and return the parsed data
pub fn fetch_ip_data() -> crate::error::Result<Vec<IpNet>> {
    let cache = Cache::new("apnic", Duration::from_secs(7 * 24 * 60 * 60));
    if let Some(data) = cache.load()? {
        info!("Loading data from cache ...");
        return Ok(parse_ip_data(
            &String::from_utf8(data).expect("The cache file should be valid UTF-8."),
        ));
    }
    info!("Fetching data from apnic.net ...");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let url = "https://ftp.apnic.net/apnic/stats/apnic/delegated-apnic-latest";
    let data = client.get(url).send().map(|r| r.text());
    match data {
        Ok(Ok(data)) => {
            info!("Fetching data from apnic.net done");
            cache.save_str(&data)?;
            Ok(parse_ip_data(&data))
        }
        // If the data fetch failed, use the built-in data instead.
        Ok(Err(e)) | Err(e) => {
            warn!("Fetching data from apnic.net failed, use built-in apnic data: {e:?}");
            let compressed_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/apnic.zst"));
            let de = zstd::stream::decode_all(Cursor::new(compressed_bytes)).unwrap();
            cache.save(&de)?;
            Ok(parse_ip_data(
                &String::from_utf8(de).expect("The cache file should be valid UTF-8."),
            ))
        }
    }
}

/// Parse IP data from str.
pub fn parse_ip_data(content: &str) -> Vec<IpNet> {
    content
        .lines()
        .map(|line| line.split('|').collect::<Vec<&str>>())
        .filter(|item| item.len() >= 5)
        .filter(|item| item[0] == "apnic" && item[1] == "CN" && ["ipv4", "ipv6"].contains(&item[2]))
        .map(|item| {
            let prefix_len = if item[2] == "ipv4" {
                32 - (item[4].parse::<u32>().expect("item[4] must be a number") as f32).log2() as u8
            } else {
                item[4].parse::<u8>().expect("item[4] must be a number")
            };
            IpNet::new(IpAddr::from_str(item[3]).unwrap(), prefix_len).unwrap()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;

    use super::*;

    #[test]
    fn test_fetch_ip_data() {
        _ = pretty_env_logger::formatted_builder()
            .filter_level(LevelFilter::Debug)
            .format_timestamp_secs()
            .filter_module("reqwest", LevelFilter::Info)
            .parse_default_env()
            .try_init();
        assert!(fetch_ip_data().is_ok());
    }

    #[test]
    fn test_parse_ip_data() {
        let results = parse_ip_data(
            std::fs::read_to_string("tests_assets/apnic.txt")
                .unwrap()
                .as_str(),
        );
        assert_eq!(results[0], IpNet::from_str("1.0.1.0/24").unwrap());
    }
}
