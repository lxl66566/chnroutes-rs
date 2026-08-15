use ipnet::IpNet;

pub mod apnic;

/// Choose a source to generate ip map.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "bin", derive(clap::ValueEnum))]
pub enum Source {
    #[default]
    Apnic,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apnic => f.write_str("apnic"),
        }
    }
}

impl Source {
    pub async fn get_cn_ips(&self) -> crate::error::Result<Vec<IpNet>> {
        match self {
            Self::Apnic => apnic::fetch_ip_data().await,
        }
    }
}
