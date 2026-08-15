use ipnet::IpNet;

pub mod apnic;

/// Choose a source to generate ip map.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Source {
    #[default]
    Apnic,
}

impl Source {
    pub async fn get_cn_ips(&self) -> crate::error::Result<Vec<IpNet>> {
        match self {
            Self::Apnic => apnic::fetch_ip_data().await,
        }
    }
}
