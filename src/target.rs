use std::{path::PathBuf, str::FromStr};

use ipnet::Ipv4Net;

use crate::{source::Source, Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    OpenVPN(u32),
    Linux,
    Mac,
    Windows(u32),
    Android,
}

impl Target {
    /// Export config string/scripts of the target.
    ///
    /// # Returns
    ///
    /// OpenVPN => Append the content to openvpn config file.
    /// Others => Return two Strings representing the upscript and downscript.
    pub fn export_str(&self, source: &Source) -> crate::error::Result<(String, Option<String>)> {
        let source_ips = source.get_cn_ips()?;
        match self {
            Self::OpenVPN(metric) => Ok((export_openvpn(source_ips, metric), None)),
            Self::Linux => Ok(export_linux(source_ips)),
            Self::Mac => Ok(export_mac(source_ips)),
            Self::Windows(metric) => Ok(export_windows(source_ips, metric)),
            Self::Android => Ok(export_android(source_ips)),
        }
    }

    /// write the export script as file to current dir.
    pub fn export_file(&self, source: &Source) -> crate::error::Result<()> {
        let (up, down) = self.export_str(source)?;
        match self {
            Self::OpenVPN(_) => std::fs::write("openvpn_conf.txt", up)?,
            other => {
                let mut up_filename = PathBuf::from("up");
                let mut down_filename = PathBuf::from("down");
                let extension = match other {
                    Self::Windows(_) => "bat",
                    _ => "sh",
                };
                up_filename.set_extension(extension);
                down_filename.set_extension(extension);

                std::fs::write(up_filename, up)?;
                if let Some(down) = down {
                    std::fs::write(down_filename, down)?;
                }
            }
        }
        Ok(())
    }
}

impl FromStr for Target {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "openvpn" => Ok(Self::OpenVPN(1)),
            "linux" => Ok(Self::Linux),
            "mac" => Ok(Self::Mac),
            "windows" => Ok(Self::Windows(1)),
            "android" => Ok(Self::Android),
            _ => Err(Error::InvalidTarget),
        }
    }
}

fn export_openvpn(ips: Vec<Ipv4Net>, metric: &u32) -> String {
    ips.into_iter()
        .map(|ip| {
            format!(
                "route {} {} net_gateway {}",
                ip.addr(),
                ip.netmask(),
                metric
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn export_linux(ips: Vec<Ipv4Net>) -> (String, Option<String>) {
    let mut up = r#"#!/bin/bash
export PATH="/bin:/sbin:/usr/sbin:/usr/bin"

OLDGW=`ip route show | grep '^default' | sed -e 's/default via \([^ ]*\).*/\1/'`

if [ $OLDGW == '' ]; then
    exit 0
fi

if [ ! -e /tmp/vpn_oldgw ]; then
    echo $OLDGW > /tmp/vpn_oldgw
fi

"#
    .to_string();
    let mut down = r#"#!/bin/bash
export PATH="/bin:/sbin:/usr/sbin:/usr/bin"

OLDGW=`cat /tmp/vpn_oldgw`

"#
    .to_string();

    up.push_str(
        ips.iter()
            .map(|ip| {
                format!(
                    "route add -net {} netmask {} gw $OLDGW",
                    ip.addr(),
                    ip.netmask(),
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    down.push_str(
        ips.iter()
            .map(|ip| format!("route del -net {} netmask {}", ip.addr(), ip.netmask(),))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    (up, Some(down))
}

fn export_mac(ips: Vec<Ipv4Net>) -> (String, Option<String>) {
    let mut up = r#"export PATH="/bin:/sbin:/usr/sbin:/usr/bin"
    
OLDGW=`netstat -nr | grep '^default' | grep -v 'ppp' | sed 's/default *\([0-9\.]*\) .*/\1/' | awk '{if($1){print $1}}'`

if [ ! -e /tmp/pptp_oldgw ]; then
    echo "${OLDGW}" > /tmp/pptp_oldgw
fi

dscacheutil -flushcache

route add 10.0.0.0/8 "${OLDGW}"
route add 172.16.0.0/12 "${OLDGW}"
route add 192.168.0.0/16 "${OLDGW}"

"#.to_string();

    let mut down = r#"#!/bin/sh
export PATH="/bin:/sbin:/usr/sbin:/usr/bin"

if [ ! -e /tmp/pptp_oldgw ]; then
        exit 0
fi

OLDGW=`cat /tmp/pptp_oldgw`

route delete 10.0.0.0/8 "${OLDGW}"
route delete 172.16.0.0/12 "${OLDGW}"
route delete 192.168.0.0/16 "${OLDGW}"

"#
    .to_string();

    up.push_str(
        ips.iter()
            .map(|ip| format!(r#"route add {} "${{OLDGW}}""#, ip))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    down.push_str(
        ips.iter()
            .map(|ip| format!(r#"route delete {} ${{OLDGW}}"#, ip))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    (up, Some(down))
}

fn export_windows(ips: Vec<Ipv4Net>, metric: &u32) -> (String, Option<String>) {
    let mut up = r#"@echo off
for /F "tokens=3" %%* in ('route print ^| findstr "\<0.0.0.0\>"') do set "gw=%%*"
echo gw=%gw%
ipconfig /flushdns

"#
    .to_string();
    let mut down = r#"@echo off

"#
    .to_string();

    up.push_str(
        ips.iter()
            .map(|ip| {
                format!(
                    r#"route add {} mask {} {} metric {}"#,
                    ip.addr(),
                    ip.netmask(),
                    "%gw%",
                    metric
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    down.push_str(
        ips.iter()
            .map(|ip| format!(r#"route delete {}"#, ip.addr()))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    (up, Some(down))
}

fn export_android(ips: Vec<Ipv4Net>) -> (String, Option<String>) {
    let mut up = r#"#!/bin/sh
alias nestat='/system/xbin/busybox netstat'
alias grep='/system/xbin/busybox grep'
alias awk='/system/xbin/busybox awk'
alias route='/system/xbin/busybox route'

OLDGW=`netstat -rn | grep ^0\.0\.0\.0 | awk '{print $2}'`

"#
    .to_string();
    let mut down = r#"#!/bin/sh
alias route='/system/xbin/busybox route'
    
"#
    .to_string();

    up.push_str(
        ips.iter()
            .map(|ip| {
                format!(
                    r#"route add -net {} netmask {} gw $OLDGW"#,
                    ip.addr(),
                    ip.netmask(),
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    down.push_str(
        ips.iter()
            .map(|ip| format!(r#"route del -net {} netmask {}"#, ip.addr(), ip.netmask()))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    (up, Some(down))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_export_file() {
        Target::Windows(123).export_file(&Source::test).unwrap();
        let up = Path::new("up.bat");
        let down = Path::new("down.bat");
        assert!(up.exists() && down.exists());
    }
}
