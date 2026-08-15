use std::{path::Path, str::FromStr};

use ipnet::IpNet;

use crate::{Error, Result, source::Source};

/// The Targets for exporting scripts, like chnroutes.py do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    OpenVPN(u32),
    Linux,
    Mac,
    Windows,
    Android,
}

impl Target {
    /// Export config string/scripts of the target.
    ///
    /// # Returns
    ///
    /// OpenVPN => Append the content to openvpn config file.
    /// Others => Return two Strings representing the upscript and downscript.
    pub async fn export_str(&self, source: &Source) -> Result<(String, Option<String>)> {
        let source_ips = source.get_cn_ips().await?;
        match self {
            Self::OpenVPN(metric) => Ok((export_openvpn(source_ips, *metric), None)),
            Self::Linux => Ok(export_linux(&source_ips)),
            Self::Mac => Ok(export_mac(&source_ips)),
            Self::Windows => Ok(export_windows(&source_ips)),
            Self::Android => Ok(export_android(&source_ips)),
        }
    }

    /// write the export script as file to current dir.
    pub async fn export_file(&self, source: &Source) -> Result<()> {
        let (up, down) = self.export_str(source).await?;
        self.write_scripts(Path::new("."), &up, down.as_deref())
    }

    fn write_scripts(self, dir: &Path, up: &str, down: Option<&str>) -> Result<()> {
        match self {
            Self::OpenVPN(_) => std::fs::write(dir.join("openvpn_conf.txt"), up)?,
            other => {
                let extension = match other {
                    Self::Windows => "bat",
                    _ => "sh",
                };
                std::fs::write(dir.join(format!("up.{extension}")), up)?;
                if let Some(down) = down {
                    std::fs::write(dir.join(format!("down.{extension}")), down)?;
                }
            },
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
            "windows" => Ok(Self::Windows),
            "android" => Ok(Self::Android),
            _ => Err(Error::InvalidTarget),
        }
    }
}

fn export_openvpn(ips: Vec<IpNet>, metric: u32) -> String {
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

fn export_linux(ips: &[IpNet]) -> (String, Option<String>) {
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
            .map(|ip| format!("route del -net {} netmask {}", ip.addr(), ip.netmask()))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    (up, Some(down))
}

fn export_mac(ips: &[IpNet]) -> (String, Option<String>) {
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
            .map(|ip| format!(r#"route add {ip} "${{OLDGW}}""#))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    down.push_str(
        ips.iter()
            .map(|ip| format!(r"route delete {ip} ${{OLDGW}}"))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    (up, Some(down))
}

fn export_windows(ips: &[IpNet]) -> (String, Option<String>) {
    let mut up = r#"@echo off
for /F "tokens=3" %%* in ('route print ^| findstr "\<0.0.0.0\>"') do set "gw=%%*"
echo gw=%gw%
ipconfig /flushdns

"#
    .to_string();
    let mut down = r"@echo off

"
    .to_string();

    up.push_str(
        ips.iter()
            .map(|ip| {
                format!(
                    r"route add {} mask {} {} metric 1",
                    ip.addr(),
                    ip.netmask(),
                    "%gw%",
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    down.push_str(
        ips.iter()
            .map(|ip| format!(r"route delete {}", ip.addr()))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    (up, Some(down))
}

fn export_android(ips: &[IpNet]) -> (String, Option<String>) {
    let mut up = r"#!/bin/sh
alias nestat='/system/xbin/busybox netstat'
alias grep='/system/xbin/busybox grep'
alias awk='/system/xbin/busybox awk'
alias route='/system/xbin/busybox route'

OLDGW=`netstat -rn | grep ^0\.0\.0\.0 | awk '{print $2}'`

"
    .to_string();
    let mut down = r"#!/bin/sh
alias route='/system/xbin/busybox route'
    
"
    .to_string();

    up.push_str(
        ips.iter()
            .map(|ip| {
                format!(
                    r"route add -net {} netmask {} gw $OLDGW",
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
            .map(|ip| format!(r"route del -net {} netmask {}", ip.addr(), ip.netmask()))
            .collect::<Vec<String>>()
            .join("\n")
            .as_str(),
    );
    (up, Some(down))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_scripts() {
        let dir = std::env::temp_dir().join(format!("chnroutes-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Target::Windows
            .write_scripts(&dir, "up-content", Some("down-content"))
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("up.bat")).unwrap(),
            "up-content"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("down.bat")).unwrap(),
            "down-content"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
