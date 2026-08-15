use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use futures_util::{StreamExt, stream};
use ipnet::IpNet;
use log::{error, info};
use net_route::{Handle, Route};
use netdev::get_default_interface;

use crate::error::RouteOpError;

/// Maximum number of route table operations in flight at once.
const MAX_CONCURRENT_ROUTE_OPS: usize = 64;

type Result<T> = std::result::Result<T, RouteOpError>;

/// The kind of a route table operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RouteAction {
    Add,
    Delete,
}

impl RouteAction {
    fn verb(self) -> &'static str {
        match self {
            Self::Add => "Adding",
            Self::Delete => "Removing",
        }
    }

    /// Already-satisfied entries are idempotent no-ops, not real failures.
    fn is_ignorable(self, err: &RouteOpError) -> bool {
        matches!(
            (self, err),
            (Self::Add, RouteOpError::RouteAlreadyExistsError)
                | (Self::Delete, RouteOpError::RouteNotFoundError)
        )
    }
}

fn default_interface_index() -> Result<u32> {
    get_default_interface()
        .map(|x| x.index)
        .map_err(RouteOpError::GetInterfaceError)
}

/// Get default gateway ipv4 and ipv6 of the system default interface.
pub async fn get_gateway(handle: &Handle) -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>)> {
    resolve_gateways(handle, default_interface_index()?).await
}

/// Resolve the default gateways among the default routes of the given interface.
async fn resolve_gateways(
    handle: &Handle,
    ifindex: u32,
) -> Result<(Option<Ipv4Addr>, Option<Ipv6Addr>)> {
    let (mut v4, mut v6) = (None, None);
    let routes = handle
        .list()
        .await?
        .into_iter()
        // Only consider default routes on the default interface: with a VPN
        // connected, other interfaces carry their own default routes whose
        // gateways must not be picked here.
        .filter(|r| r.destination.is_unspecified() && r.ifindex == Some(ifindex))
        .filter_map(|r| r.gateway);
    for ip in routes {
        match ip {
            IpAddr::V4(ipv4) if v4.is_none() => v4 = Some(ipv4),
            IpAddr::V6(ipv6) if v6.is_none() => v6 = Some(ipv6),
            _ => {},
        }
        if v4.is_some() && v6.is_some() {
            return Ok((v4, v6));
        }
    }
    Ok((v4, v6))
}

/// Shared context for route table operations: the OS handle, the default
/// interface index, and the default gateways (unresolved for deletions).
pub struct RouteContext<'a> {
    handle: &'a Handle,
    ifindex: u32,
    gateways: (Option<Ipv4Addr>, Option<Ipv6Addr>),
}

impl RouteContext<'_> {
    /// Create a context for adding routes, resolving the default gateways.
    pub async fn for_add(handle: &Handle) -> Result<RouteContext<'_>> {
        let ifindex = default_interface_index()?;
        Ok(RouteContext {
            handle,
            ifindex,
            gateways: resolve_gateways(handle, ifindex).await?,
        })
    }

    /// Create a context for deleting routes; gateways are not needed here.
    pub fn for_delete(handle: &Handle) -> Result<RouteContext<'_>> {
        Ok(RouteContext {
            handle,
            ifindex: default_interface_index()?,
            gateways: (None, None),
        })
    }

    /// Add one route entry to routing table.
    pub async fn add(&self, route: &IpNet) -> Result<()> {
        let (v4, v6) = self.gateways;
        let route_item = &Route::new(route.addr(), route.prefix_len())
            // deal with ipv4 and ipv6
            .with_gateway(if route.addr().is_ipv4() {
                IpAddr::from(v4.ok_or(RouteOpError::NoGatewayError)?)
            } else {
                #[cfg(not(windows))]
                {
                    IpAddr::from(v6.ok_or(RouteOpError::NoGatewayError)?)
                }
                #[cfg(windows)]
                {
                    // on windows, gateway can be ipv4 while destination is ipv6
                    if let Some(g) = v6 {
                        IpAddr::from(g)
                    } else if let Some(g) = v4 {
                        IpAddr::from(g)
                    } else {
                        return Err(RouteOpError::NoGatewayError);
                    }
                }
            })
            .with_ifindex(self.ifindex);

        self.handle
            .add(route_item)
            .await
            .map_err(classify_add_error)
    }

    /// Delete route entry from routing table.
    pub async fn delete(&self, route: &IpNet) -> Result<()> {
        let route_item = &Route::new(route.addr(), route.prefix_len()).with_ifindex(self.ifindex);
        self.handle
            .delete(route_item)
            .await
            .map_err(classify_delete_error)
    }
}

/// net-route surfaces OS failures as plain `std::io::Error`s, so map the
/// well-known cases onto dedicated error variants.
fn classify_add_error(err: std::io::Error) -> RouteOpError {
    if is_already_exists(&err) {
        return RouteOpError::RouteAlreadyExistsError;
    }
    err.into()
}

fn is_already_exists(err: &std::io::Error) -> bool {
    // Linux netlink EEXIST (17), Windows ERROR_OBJECT_ALREADY_EXISTS (5010).
    if matches!(err.raw_os_error(), Some(17 | 5010)) {
        return true;
    }
    // Received a netlink error message File exists (os error 17)
    // We cannot use other way to deal with this error because net-route crate turns
    // it into std::io::Error with no more information.
    err.kind() == std::io::ErrorKind::Other && err.to_string().contains("exists")
}

fn classify_delete_error(err: std::io::Error) -> RouteOpError {
    // Linux netlink ESRCH (3), Windows ERROR_NOT_FOUND (1168).
    if err.kind() == std::io::ErrorKind::NotFound || matches!(err.raw_os_error(), Some(3 | 1168)) {
        return RouteOpError::RouteNotFoundError;
    }
    err.into()
}

/// Add one route entry to routing table.
pub async fn add_route(handle: &Handle, route: &IpNet) -> Result<()> {
    RouteContext::for_add(handle).await?.add(route).await
}

/// Delete route entry from routing table.
pub async fn del_route(handle: &Handle, route: &IpNet) -> Result<()> {
    RouteContext::for_delete(handle)?.delete(route).await
}

/// Apply multiple routes to the routing table.
///
/// Individual failures do not abort the batch: already-satisfied entries are
/// counted as ignored, other errors are logged, and the batch returns
/// [`RouteOpError::ApplyError`] if any route failed.
async fn apply_routes(routes: &[IpNet], action: RouteAction) -> Result<()> {
    info!("{} {} routes...", action.verb(), routes.len());
    let handle = Handle::new().map_err(|_| RouteOpError::HandleInitError)?;
    let ctx = match action {
        RouteAction::Add => RouteContext::for_add(&handle).await?,
        RouteAction::Delete => RouteContext::for_delete(&handle)?,
    };

    let mut applied = 0;
    let mut ignored = 0;
    let mut failed = 0;
    let mut stream = stream::iter(routes.iter().map(|route| {
        let ctx = &ctx;
        async move {
            let result = match action {
                RouteAction::Add => ctx.add(route).await,
                RouteAction::Delete => ctx.delete(route).await,
            };
            (route, result)
        }
    }))
    .buffer_unordered(MAX_CONCURRENT_ROUTE_OPS);

    while let Some((route, result)) = stream.next().await {
        match result {
            Ok(()) => applied += 1,
            Err(err) if action.is_ignorable(&err) => ignored += 1,
            Err(err) => {
                failed += 1;
                error!("Error while applying route {route}: {err:?}");
            },
        }
    }
    info!("Routes applied: {applied}, ignored: {ignored}, failed: {failed}.");
    if failed > 0 {
        return Err(RouteOpError::ApplyError { failed });
    }
    Ok(())
}

/// Add multiple routes to routing table.
pub async fn add_routes(routes: &[IpNet]) -> Result<()> {
    apply_routes(routes, RouteAction::Add).await
}

/// Delete multiple routes from routing table.
pub async fn del_routes(routes: &[IpNet]) -> Result<()> {
    apply_routes(routes, RouteAction::Delete).await
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

        assert!(
            handle
                .list()
                .await
                .unwrap()
                .into_iter()
                .any(|r| r.destination == dest)
        );

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
