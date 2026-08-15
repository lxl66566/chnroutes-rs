use chnroutes::{Result, Source, Target};

#[allow(unused)]
#[tokio::main]
async fn main() -> Result<()> {
    /// Get the CN IPs from APNIC
    let cn_ip_results: Vec<ipnet::IpNet> = chnroutes::source::apnic::fetch_ip_data().await?;
    /// Get the user script
    let user_script: Result<(String, Option<String>)> =
        Target::Linux.export_str(&Source::apnic).await;
    /// Apply rules to system route table
    chnroutes::up(&Source::default()).await?;
    Ok(())
}
