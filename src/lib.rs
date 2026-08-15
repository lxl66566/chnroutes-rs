pub mod cache;
pub mod error;
pub mod route_op;
pub mod source;
pub mod target;

pub use error::{Error, Result};
pub use source::Source;
pub use target::Target;

pub async fn up(source: &Source) -> Result<()> {
    route_op::add_routes(&source.get_cn_ips().await?).await?;
    Ok(())
}

pub async fn down(source: &Source) -> Result<()> {
    route_op::del_routes(&source.get_cn_ips().await?).await?;
    Ok(())
}
