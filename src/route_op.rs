use std::{net::IpAddr, sync::OnceLock};

use futures_util::{stream::FuturesUnordered, TryStreamExt};
use ipnet::IpNet;
use net_route::{Handle, Route};
use pollster::FutureExt;
use tap::Tap;

use crate::error::RouteOpError;
pub static GATEWAY: OnceLock<IpAddr> = OnceLock::new();
pub static INTERFACE_INDEX: OnceLock<u32> = OnceLock::new();
use log::{error, info};
use netdev::get_default_interface;

type Result<T> = std::result::Result<T, RouteOpError>;

/// Get default gateway ip.
pub async fn get_gateway(handle: &Handle) -> Result<IpAddr> {
    let routes = handle.list().await?;
    Ok(routes
        .into_iter()
        .filter(|r| r.gateway.is_some())
        .find(|r| r.destination.is_unspecified())
        .ok_or(RouteOpError::NoGatewayError)?
        .gateway
        .unwrap())
}

/// Add ont route entry to routing table.
pub async fn add_route(handle: &Handle, route: &IpNet) -> Result<()> {
    handle
        .add(
            &Route::new(route.addr(), route.prefix_len())
                .with_gateway(*GATEWAY.get_or_try_init(|| get_gateway(handle).block_on())?)
                .with_ifindex(
                    *INTERFACE_INDEX
                        .get_or_try_init(|| get_default_interface().map(|x| x.index))
                        .map_err(RouteOpError::GetInterfaceError)?,
                ),
        )
        .await?;
    Ok(())
}

/// Delete route entry from routing table.
pub async fn del_route(handle: &Handle, route: &IpNet) -> Result<()> {
    handle
        .delete(
            &Route::new(route.addr(), route.prefix_len()).with_ifindex(
                *INTERFACE_INDEX
                    .get_or_try_init(|| get_default_interface().map(|x| x.index))
                    .map_err(RouteOpError::GetInterfaceError)?,
            ),
        )
        .await?;
    Ok(())
}

/// Add multiple routes to routing table.
pub async fn add_routes(routes: &[IpNet]) -> Result<()> {
    info!("Adding {} routes...", routes.len());
    let handle = Box::leak(Box::new(
        Handle::new().map_err(|_| RouteOpError::HandleInitError)?,
    ));
    let mut futures = routes
        .iter()
        .map(|r| add_route(handle, r))
        .collect::<FuturesUnordered<_>>();
    let mut num = 1;
    while (futures.try_next().await.tap(|res| {
        if res.is_err() {
            error!(
                "Add {} th route error: {:?}",
                num,
                res.as_ref().unwrap_err()
            );
        }
    })?)
    .is_some()
    {
        num += 1;
    }
    info!("Add success.");
    Ok(())
}

/// Delete multiple routes from routing table.
pub async fn del_routes(routes: &[IpNet]) -> Result<()> {
    info!("Removing {} routes...", routes.len());
    let handle = Handle::new().map_err(|_| RouteOpError::HandleInitError)?;
    let mut futures = routes
        .iter()
        .map(|r| del_route(&handle, r))
        .collect::<FuturesUnordered<_>>();
    let mut num = 1;
    while (futures.try_next().await.tap(|res| {
        if res.is_err() {
            error!(
                "Delete {} th route error: {:?}",
                num,
                res.as_ref().unwrap_err()
            );
        }
    })?)
    .is_some()
    {
        num += 1;
    }
    info!("Remove success.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gateway() {
        let handle = Handle::new().unwrap();
        let result = get_gateway(&handle).block_on();
        assert!(result.is_ok());
        dbg!(result.unwrap());
    }

    fn test_add_remove_route(dest: IpAddr) {
        let handle = Handle::new().unwrap();
        let _ = del_route(&handle, &IpNet::new(dest, 32).unwrap()).block_on();

        add_route(&handle, &IpNet::new(dest, 32).unwrap())
            .block_on()
            .unwrap();

        assert!(handle
            .list()
            .block_on()
            .unwrap()
            .into_iter()
            .any(|r| r.destination == dest));

        del_route(&handle, &IpNet::new(dest, 32).unwrap())
            .block_on()
            .unwrap();
    }

    #[test]
    #[ignore = "Test this as Administrator."]
    fn test_add_remove_route_v6() {
        test_add_remove_route("2001:253::".parse::<IpAddr>().unwrap());
    }

    #[test]
    #[ignore = "Test this as Administrator."]
    fn test_add_remove_route_v4() {
        test_add_remove_route(IpAddr::from([123, 123, 123, 123]));
    }
}
