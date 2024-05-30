use chnroutes::{Result, Source, Target};

fn main() -> Result<()> {
    let cn_ip_results: Vec<ipnet::Ipv4Net> = chnroutes::source::apnic::fetch_ip_data()?;
    let user_script: Result<(String, Option<String>)> = Target::Linux.export_str(&Source::apnic);
    Ok(())
}
