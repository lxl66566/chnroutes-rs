use std::{io, process::Command};

use crate::{error::Result, source::Source, target::Target};

#[cfg(windows)]
fn exec(command: &str) -> std::io::Result<String> {
    let output = Command::new("cmd.exe").args(["/C", command]).output()?;
    // TODO: handle encoding.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("Err: {}", stderr);
        return Err(io::Error::new(io::ErrorKind::Other, stderr));
    } else if !stdout.is_empty() {
        println!("{}", stdout);
        return Ok(stdout.into_owned());
    }
    Ok("".into())
}

#[cfg(unix)]
fn exec(command: &str) -> std::io::Result<String> {
    let output = Command::new("sh").arg("-c").arg(command).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("Err: {}", stderr);
        return Err(io::Error::new(io::ErrorKind::Other, stderr));
    } else if !stdout.is_empty() {
        println!("{}", stdout);
        return Ok(stdout.into_owned());
    }
    Ok("".into())
}

fn get_commands(source: &Source) -> Result<(String, Option<String>)> {
    if cfg!(target_os = "windows") {
        Target::Windows(1).export_str(source)
    } else if cfg!(target_os = "macos") {
        Target::Mac.export_str(source)
    } else if cfg!(target_os = "android") {
        Target::Android.export_str(source)
    } else {
        Target::Linux.export_str(source)
    }
}

/// Add routes to the system routing table.
pub fn up(source: &Source) -> Result<()> {
    println!("Adding routes, please wait ...");
    let commands = get_commands(source)?;
    for (index, command) in commands.0.lines().enumerate() {
        let command = command.trim();
        #[cfg(debug_assertions)]
        match index {
            x if x <= 10 => println!("exec `{}`", command),
            11 => println!("And more ..."),
            _ => {
                exec(command)?;
            }
        }
        #[cfg(not(debug_assertions))]
        exec(command)?;
    }
    Ok(())
}

/// Remove routes from the system routing table.
pub fn down(source: &Source) -> Result<()> {
    println!("Removing routes, please wait ...");
    let commands = get_commands(source)?;
    for (index, command) in commands
        .1
        .expect("The target should have a downscript")
        .lines()
        .enumerate()
    {
        let command = command.trim();
        #[cfg(debug_assertions)]
        match index {
            x if x <= 10 => println!("exec `{}`", command),
            11 => println!("And more ..."),
            _ => {
                exec(command)?;
            }
        }
        #[cfg(not(debug_assertions))]
        exec(command)?;
    }
    Ok(())
}
