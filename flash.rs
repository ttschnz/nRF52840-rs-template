#!/usr/bin/env rust-script

//! ```cargo
//! [dependencies]
//! wsl = "0.1.0"
//! ```
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{fs, process, thread};
use std::fs::File;
use std::io;

/// Copy file contents only (no chmod), so FAT/UF2 drives don't reject it.
fn copy_contents(src: &str, dst: &Path) -> io::Result<()> {
    let mut from = File::open(src)?;
    let mut to = File::create(dst)?;
    io::copy(&mut from, &mut to)?;
    to.sync_all().ok(); // the bootloader may reset before this completes; ignore errors
    Ok(())
}
/// Run a command, inheriting stdio, and abort the script if it fails.
fn exec(cmd: &str, args: &[&str]) {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("failed to run {cmd}: {e}");
            process::exit(1);
        });
    if !status.success() {
        eprintln!("{cmd} exited with {status}");
        process::exit(status.code().unwrap_or(1));
    }
}

/// Run a command silently and return whether it succeeded.
fn succeeds(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn sleep(secs: u64) {
    thread::sleep(Duration::from_secs(secs));
}

/// Where is the filesystem with this label mounted, if anywhere?
fn find_mountpoint(label: &str) -> Option<String> {
    let out = Command::new("findmnt")
        .args(["-rn", "-S", &format!("LABEL={label}"), "-o", "TARGET"])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();
    if s.is_empty() { None } else { Some(s) }
}

/// Wait for the labelled device, then return a mounted path for it.
fn wait_and_mount_linux(label: &str) -> String {
    let dev = format!("/dev/disk/by-label/{label}");
    println!("Waiting for {label} ...");
    while !Path::new(&dev).exists() {
        sleep(1);
    }

    // Give a desktop automounter a moment to do its thing first.
    for _ in 0..3 {
        if let Some(m) = find_mountpoint(label) {
            return m;
        }
        sleep(1);
    }

    // Not mounted: try udisks (no root needed)...
    if succeeds("udisksctl", &["mount", "-b", &dev]) {
        if let Some(m) = find_mountpoint(label) {
            return m;
        }
    }

    // ...then fall back to a plain mount.
    let dir = format!("/mnt/{}", label.to_lowercase());
    exec("sudo", &["mkdir", "-p", &dir]);
    exec("sudo", &["mount", &dev, &dir]);
    dir
}

/// True once Windows reports that D:\ exists.
fn windows_drive_present(letter: char) -> bool {
    let out = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Test-Path {letter}:\\"),
        ])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim() == "True",
        Err(_) => false,
    }
}

#[cfg(target_os = "linux")]
fn main() {

    let elf = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: flash.rs <elf>");
        process::exit(1);
    });

    exec("llvm-objcopy", &["-O", "binary", &elf, "nrf.bin"]);
    exec(
        "uf2conv",
        &[
            "nrf.bin", "--base", "0x27000", "--family", "0xADA52840", "--output", "nrf.uf2",
        ],
    );
    let _ = fs::remove_file("nrf.bin");

    let dest_dir: &str = if wsl::is_wsl() {
        let mount = "/mnt/d";

        // Drop any stale mount from a previous run (harmless if nothing is mounted).
        // Must come before mkdir, which stats the path and fails on a dead mount.
        let _ = succeeds("sudo", &["umount", "-l", mount]);

        println!("Waiting for D: ...");
        while !windows_drive_present('D') {
            sleep(2);
        }

        exec("sudo", &["mkdir", "-p", mount]);
        while !succeeds("sudo", &["mount", "-t", "drvfs", "D:", mount]) {
            sleep(2);
        }
        println!("D: mounted at {mount}");
        mount
    } else {
        &wait_and_mount_linux("XIAO-SENSE")
    };

    // mv nrf.uf2 <drive>/  (copy + delete, since rename can't cross filesystems)
    let target = Path::new(&dest_dir).join("nrf.uf2");
    if let Err(e) = copy_contents("nrf.uf2", &target) {
        eprintln!("copy failed: {e}");
        process::exit(1);
    }
    let _ = fs::remove_file("nrf.uf2");

    println!("done");
}

#[cfg(not(target_os = "linux"))]
fn main(){
    println!("flash script not implemented outside of linux. Use WSL if on Windows.");
}
