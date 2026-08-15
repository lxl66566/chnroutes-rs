use std::{io::Cursor, net::IpAddr, str::FromStr, time::Duration};

use ipnet::IpNet;
use log::{info, warn};

use crate::{cache::Cache, error::CacheError};

/// Fetch IP data from apnic.net, add it to cache and return the parsed data
pub async fn fetch_ip_data() -> crate::error::Result<Vec<IpNet>> {
    let cache = Cache::new("apnic", Duration::from_secs(7 * 24 * 60 * 60));
    if let Some(data) = load_cache(&cache)? {
        info!("Loading data from cache ...");
        return Ok(parse_ip_data(&data));
    }
    info!("Fetching data from apnic.net ...");
    match fetch_remote_data().await {
        Ok(data) => {
            info!("Fetching data from apnic.net done");
            cache.save_str(&data)?;
            Ok(parse_ip_data(&data))
        },
        // If the data fetch failed, use the built-in data instead.
        Err(e) => {
            warn!("Fetching data from apnic.net failed, use built-in apnic data: {e:?}");
            let compressed_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/apnic.zst"));
            let de = zstd::stream::decode_all(Cursor::new(compressed_bytes))?;
            cache.save(&de)?;
            Ok(parse_ip_data(
                &String::from_utf8(de).expect("The built-in data should be valid UTF-8."),
            ))
        },
    }
}

async fn fetch_remote_data() -> reqwest::Result<String> {
    let url = "https://ftp.apnic.net/apnic/stats/apnic/delegated-apnic-latest";
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?
        .get(url)
        .send()
        .await?
        .text()
        .await
}

/// Load the cached source file, treating a corrupted (non UTF-8) cache as a miss.
fn load_cache(cache: &Cache) -> Result<Option<String>, CacheError> {
    let Some(bytes) = cache.load()? else {
        return Ok(None);
    };
    if let Ok(text) = String::from_utf8(bytes) {
        return Ok(Some(text));
    }
    warn!("Cache file is corrupted, removing it and refetching ...");
    cache.remove()?;
    Ok(None)
}

/// Parse IP data from str.
///
/// Only `apnic|CN|ipv4/ipv6` records are kept. Malformed records are skipped
/// with a warning instead of aborting the whole parse.
#[must_use]
pub fn parse_ip_data(content: &str) -> Vec<IpNet> {
    content
        .lines()
        .map(|line| line.split('|').collect::<Vec<&str>>())
        .filter(|fields| {
            fields.len() >= 5
                && fields[0] == "apnic"
                && fields[1] == "CN"
                && matches!(fields[2], "ipv4" | "ipv6")
        })
        .filter_map(|fields| {
            parse_cn_record(&fields)
                .map_err(|reason| {
                    warn!("Skipping invalid apnic record {:?}: {reason}", fields.join("|"));
                })
                .ok()
        })
        .collect()
}

/// Parse a single `apnic|CN|...` record into an [`IpNet`].
///
/// The delegated-apnic format encodes IPv4 ranges as `start|count` and IPv6
/// ranges as `start|prefix_len`.
fn parse_cn_record(fields: &[&str]) -> Result<IpNet, String> {
    let addr = IpAddr::from_str(fields[3]).map_err(|e| format!("invalid address: {e}"))?;
    let prefix_len = match fields[2] {
        "ipv4" => {
            let count: u32 = fields[4]
                .parse()
                .map_err(|e| format!("invalid address count: {e}"))?;
            // A range whose size is not a power of two cannot be a single CIDR
            // block; `ilog2` would silently round down and widen the prefix.
            if !count.is_power_of_two() {
                return Err(format!("address count {count} is not a power of two"));
            }
            32 - u8::try_from(count.ilog2()).map_err(|e| format!("prefix overflow: {e}"))?
        },
        "ipv6" => fields[4]
            .parse::<u8>()
            .map_err(|e| format!("invalid prefix length: {e}"))?,
        kind => return Err(format!("unknown record type {kind:?}")),
    };
    IpNet::new(addr, prefix_len).map_err(|e| format!("invalid network: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_fetch_ip_data() {
        assert!(fetch_ip_data().await.is_ok());
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

    #[test]
    fn test_parse_ip_data_skips_invalid_records() {
        let content = "\
apnic|CN|ipv4|1.0.1.0|256|20110414|allocated
apnic|CN|ipv6|2001:250::|35|20040810|allocated
apnic|JP|ipv4|1.0.16.0|4096|20110414|allocated
apnic|CN|asn|9394|2|20010719|allocated
apnic|CN|ipv4|2.0.0.0|0|20110414|allocated
apnic|CN|ipv4|3.0.0.0|1000|20110414|allocated
apnic|CN|ipv4|not-an-ip|256|20110414|allocated
apnic|CN|ipv6|2001:251::|not-a-number|20040810|allocated
apnic|CN|ipv6|2001:252::|200|20040810|allocated
apnic|CN|ipv4
";
        let results = parse_ip_data(content);
        assert_eq!(
            results,
            vec![
                IpNet::from_str("1.0.1.0/24").unwrap(),
                IpNet::from_str("2001:250::/35").unwrap(),
            ]
        );
    }

    #[test]
    fn test_load_corrupted_cache() {
        let cache = Cache::new(
            format!("corrupted-{}", std::process::id()),
            Duration::from_secs(60),
        );
        cache.save(&[0xff, 0xfe]).unwrap();
        assert!(load_cache(&cache).unwrap().is_none());
        cache.remove().unwrap();
    }
}
