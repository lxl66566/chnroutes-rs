use std::{net::IpAddr, str::FromStr, time::Duration};

use ipnet::IpNet;
use log::info;

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
    let url = "https://ftp.apnic.net/apnic/stats/apnic/delegated-apnic-latest";
    let data = reqwest::blocking::get(url)?.text()?;
    cache.save_str(&data)?;
    Ok(parse_ip_data(&data))
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
    use super::*;

    #[test]
    fn test_parse_ip_data() {
        let results = parse_ip_data(
            std::fs::read_to_string("tests_assets/apnic.txt")
                .unwrap()
                .as_str(),
        );
        assert_eq!(results[0], IpNet::from_str("1.0.1.0/24").unwrap());
        assert_eq!(results[0], IpNet::from_str("1.0.2.0/23").unwrap());
    }
}
