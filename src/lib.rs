#![feature(once_cell_try)]
#![feature(lazy_cell)]
#![feature(iterator_try_collect)]
pub mod cache;
pub mod error;
pub mod route_op;
pub mod source;
pub mod target;

pub use error::{Error, Result};
use pollster::FutureExt;
pub use source::Source;
pub use target::Target;

pub fn up(source: &Source) -> Result<()> {
    Ok(route_op::add_routes(&source.get_cn_ips()?).block_on()?)
}

pub fn down(source: &Source) -> Result<()> {
    Ok(route_op::del_routes(&source.get_cn_ips()?).block_on()?)
}
