#[cfg(test)]
use std::str::FromStr;

use ipnet::IpNet;

pub mod apnic;

/// Choose a source to generate ip map.
#[derive(Debug, Clone, Copy, Default, enum_tools::EnumTools)]
#[enum_tools(as_str, from_str, iter, next)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum Source {
    #[default]
    apnic,
    #[cfg(test)]
    test,
}

impl Source {
    pub fn get_cn_ips(&self) -> crate::error::Result<Vec<IpNet>> {
        get_cn_ips(self)
    }
}

pub fn get_cn_ips(source: &Source) -> crate::error::Result<Vec<IpNet>> {
    match source {
        Source::apnic => apnic::fetch_ip_data(),
        #[cfg(test)]
        Source::test => Ok(vec![
            IpNet::from_str("1.0.1.0/24").unwrap(),
            IpNet::from_str("1.0.2.0/23").unwrap(),
        ]),
    }
}
