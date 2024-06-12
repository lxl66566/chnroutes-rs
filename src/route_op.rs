use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::OnceLock,
};

use futures_util::{stream::FuturesOrdered, TryStreamExt};
use ipnet::IpNet;
use net_route::{Handle, Route};
use tap::Tap;
use tokio::sync::OnceCell;

use crate::error::RouteOpError;
pub static GATEWAY: OnceCell<(Option<Ipv4Addr>, Option<Ipv6Addr>)> = OnceCell::const_new();
pub static INTERFACE_INDEX: OnceLock<u32> = OnceLock::new();
use log::{error, info};
use netdev::get_default_interface;

type Result<T> = std::result::Result<T, RouteOpError>;

/// Get default gateway ipv4 and ipv6.
pub async fn get_gateway(handle: &Handle) -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>)> {
    let (mut v4, mut v6) = (None, None);
    let routes = handle
        .list()
        .await?
        .into_iter()
        .filter(|r| r.gateway.is_some())
        .filter(|r| r.destination.is_unspecified())
        .filter_map(|r| r.gateway);
    for ip in routes {
        match ip {
            IpAddr::V4(ipv4) if v4.is_none() => v4 = Some(ipv4),
            IpAddr::V6(ipv6) if v6.is_none() => v6 = Some(ipv6),
            _ => {}
        }
        if v4.is_some() && v6.is_some() {
            return Ok((v4, v6));
        }
    }
    Ok((v4, v6))
}

/// Add ont route entry to routing table.
pub async fn add_route(handle: &Handle, route: &IpNet) -> Result<()> {
    let gateway = GATEWAY.get_or_try_init(|| get_gateway(handle)).await?;
    let route_item = &Route::new(route.addr(), route.prefix_len())
        .with_gateway(if route.addr().is_ipv4() {
            IpAddr::from(gateway.0.ok_or(RouteOpError::NoGatewayError)?)
        } else {
            #[cfg(not(windows))]
            {
                IpAddr::from(gateway.1.ok_or(RouteOpError::NoGatewayError)?)
            }
            #[cfg(windows)]
            {
                // on windows, gateway can be ipv4 while destination is ipv6
                if let Some(g) = gateway.1 {
                    IpAddr::from(g)
                } else if let Some(g) = gateway.0 {
                    IpAddr::from(g)
                } else {
                    return Err(RouteOpError::NoGatewayError);
                }
            }
        })
        .with_ifindex(
            *INTERFACE_INDEX
                .get_or_try_init(|| get_default_interface().map(|x| x.index))
                .map_err(RouteOpError::GetInterfaceError)?,
        );
    handle.add(route_item).await?;
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
        .collect::<FuturesOrdered<_>>();
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
        .collect::<FuturesOrdered<_>>();
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

    #[tokio::test]
    async fn test_get_gateway() {
        let handle = Handle::new().unwrap();
        let result = get_gateway(&handle).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.0.is_some() || result.1.is_some());
        dbg!(result);
    }

    async fn test_add_remove_route(dest: IpAddr) {
        let handle = Handle::new().unwrap();
        let _ = del_route(&handle, &IpNet::new(dest, 32).unwrap()).await;

        add_route(&handle, &IpNet::new(dest, 32).unwrap())
            .await
            .unwrap();

        assert!(handle
            .list()
            .await
            .unwrap()
            .into_iter()
            .any(|r| r.destination == dest));

        del_route(&handle, &IpNet::new(dest, 32).unwrap())
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "Test this as Administrator."]
    async fn test_add_remove_route_v6() {
        test_add_remove_route("2001:253::".parse::<IpAddr>().unwrap()).await;
    }

    #[tokio::test]
    #[ignore = "Test this as Administrator."]
    async fn test_add_remove_route_v4() {
        test_add_remove_route(IpAddr::from([123, 123, 123, 123])).await;
    }
}
