use chnroutes::{Result, Source, Target};

#[allow(unused)]
fn main() -> Result<()> {
    let cn_ip_results: Vec<ipnet::IpNet> = chnroutes::source::apnic::fetch_ip_data()?;
    let user_script: Result<(String, Option<String>)> = Target::Linux.export_str(&Source::apnic);
    let up_result: Result<()> = chnroutes::up(&Default::default());
    Ok(())
}
