use std::{net::Ipv4Addr, str::FromStr, time::Duration};

use ipnet::Ipv4Net;

use crate::cache::Cache;

/// Fetch IP data from apnic.net, add it to cache and return the parsed data
pub fn fetch_ip_data() -> crate::error::Result<Vec<Ipv4Net>> {
    let cache = Cache::new("apnic", Duration::from_secs(7 * 24 * 60 * 60));
    if let Some(data) = cache.load()? {
        println!("Loading data from cache ...");
        return Ok(parse_ip_data(
            &String::from_utf8(data).expect("The cache file should be valid UTF-8."),
        ));
    }
    println!("Fetching data from apnic.net ...");
    let url = "https://ftp.apnic.net/apnic/stats/apnic/delegated-apnic-latest";
    let data = reqwest::blocking::get(url)?.text()?;
    cache.save_str(&data)?;
    Ok(parse_ip_data(&data))
}

/// Parse IP data from str
pub fn parse_ip_data(content: &str) -> Vec<Ipv4Net> {
    content
        .lines()
        .map(|line| line.split('|').collect::<Vec<&str>>())
        .filter(|item| item.len() >= 5)
        .filter(|item| item[0] == "apnic" && item[1] == "CN" && item[2] == "ipv4")
        .map(|item| {
            Ipv4Net::new(
                Ipv4Addr::from_str(item[3]).unwrap(),
                32 - (item[4].parse::<u32>().unwrap() as f32).log2() as u8,
            )
            .unwrap()
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
        assert_eq!(results[0], Ipv4Net::from_str("1.0.1.0/24").unwrap());
        assert_eq!(results[0], Ipv4Net::from_str("1.0.2.0/23").unwrap());
    }
}
