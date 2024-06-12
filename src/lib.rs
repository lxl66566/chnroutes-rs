#![feature(once_cell_try)]
#![feature(iterator_try_collect)]
pub mod cache;
pub mod error;
pub mod route_op;
pub mod source;
pub mod target;

pub use error::{Error, Result};
pub use source::Source;
pub use target::Target;

pub async fn up(source: &Source) -> Result<()> {
    Ok(route_op::add_routes(&source.get_cn_ips()?).await?)
}

pub async fn down(source: &Source) -> Result<()> {
    Ok(route_op::del_routes(&source.get_cn_ips()?).await?)
}
