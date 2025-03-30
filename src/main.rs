use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use reqwest::blocking::get;
use log::{info, error};
use structopt::StructOpt;
use regex::Regex;

#[derive(StructOpt, Debug)]
#[structopt(name = "gcp-dynamic-ip-updater")]
struct Opt {
    #[structopt(name = "firewall_rule_names")]
    firewall_rule_names: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct IpAddresses {
    previous_ip: Option<String>,
    current_ip: Option<String>,
}

fn get_public_ip() -> Option<String> {
    match get("https://api.ipify.org?format=json") {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>() {
                    Ok(json) => json["ip"].as_str().map(|s| s.to_string()),
                    Err(e) => {
                        error!("Failed to parse IP address: {}", e);
                        None
                    }
                }
            } else {
                error!("Failed to fetch public IP address: HTTP {}", response.status());
                None
            }
        }
        Err(e) => {
            error!("Failed to fetch public IP address: {}", e);
            None
        }
    }
}

fn check_ip_changed(ip: &str) -> bool {
    let file_path = "./ip_addresses.json";
    let mut data = if let Ok(contents) = fs::read_to_string(file_path) {
        serde_json::from_str(&contents).unwrap_or(IpAddresses {
            previous_ip: None,
            current_ip: None,
        })
    } else {
        IpAddresses {
            previous_ip: None,
            current_ip: None,
        }
    };

    if data.current_ip.as_deref() != Some(ip) {
        info!("IP address has changed: previous_ip={:?}, current_ip={}", data.current_ip, ip);
        data.previous_ip = data.current_ip.clone();
        data.current_ip = Some(ip.to_string());

        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = fs::write(file_path, json);
        }
        true
    } else {
        false
    }
}

fn update_firewall_rule(firewall_rule_name: &str, new_ip: &str) -> io::Result<()> {
    // Validate firewall_rule_name
    let valid_name = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    if !valid_name.is_match(firewall_rule_name) {
        error!("Invalid firewall rule name: {}", firewall_rule_name);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid firewall rule name"));
    }

    // Validate new_ip
    let valid_ip = Regex::new(r"^\d{1,3}(\.\d{1,3}){3}$").unwrap();
    if !valid_ip.is_match(new_ip) {
        error!("Invalid IP address: {}", new_ip);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid IP address"));
    }

    let output = Command::new("gcloud")
        .args(&[
            "compute",
            "firewalls",
            "update",
            firewall_rule_name,
            "--source-ranges",
            &format!("{}/32", new_ip),
        ])
        .output()?;

    if output.status.success() {
        info!("Firewall rule updated: firewall_rule_name={}, ip={}", firewall_rule_name, new_ip);
    } else {
        error!(
            "Failed to update firewall rule: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    info!("Starting dynamic IP updater: firewall_rule_names={:?}", opt.firewall_rule_names);

    loop {
        if let Some(ip) = get_public_ip() {
            if check_ip_changed(&ip) {
                for firewall_rule_name in &opt.firewall_rule_names {
                    if let Err(e) = update_firewall_rule(firewall_rule_name, &ip) {
                        error!("Error updating firewall rule: {}", e);
                    }
                }
            }
        }
        sleep(Duration::from_secs(180)); // Sleep for 3 minutes
    }
}