use std::io::{self};
use std::thread::sleep;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use reqwest::blocking::get;
use log::{info, error, debug};
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
    debug!("Attempting to fetch public IP address...");
    match get("https://api.ipify.org?format=json") {
        Ok(response) => {
            debug!("Received response from IP API: {:?}", response);
            if response.status().is_success() {
                match response.json::<serde_json::Value>() {
                    Ok(json) => {
                        debug!("Parsed JSON response: {:?}", json);
                        if let Some(ip) = json["ip"].as_str() {
                            let ip_string = ip.to_string();
                            info!("Retrieved public IP address: {}", ip_string);
                            Some(ip_string)
                        } else {
                            error!("IP address not found in response");
                            None
                        }
                    }
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

fn check_ip_changed(ip: &str, ip_data: &mut IpAddresses) -> bool {
    debug!(
        "Checking if IP address has changed. Current IP: {:?}, New IP: {}",
        ip_data.current_ip, ip
    );
    if ip_data.current_ip.as_deref() != Some(ip) {
        info!(
            "IP address has changed: previous_ip={:?}, current_ip={}",
            ip_data.current_ip, ip
        );
        ip_data.previous_ip = ip_data.current_ip.clone();
        ip_data.current_ip = Some(ip.to_string());
        true
    } else {
        debug!("IP address has not changed.");
        false
    }
}

async fn update_firewall_rule(firewall_rule_name: &str, new_ip: &str) -> io::Result<()> {
    debug!(
        "Updating firewall rule: firewall_rule_name={}, new_ip={}",
        firewall_rule_name, new_ip
    );

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

    debug!("Executing gcloud command to update firewall rule...");
    let client = google_cloud::Client::default();
    let firewall = client
        .compute()
        .firewalls()
        .get(firewall_rule_name)
        .await
        .map_err(|e| {
            error!("Failed to retrieve firewall rule: {}", e);
            io::Error::new(io::ErrorKind::Other, "Failed to retrieve firewall rule")
        })?;

    let mut firewall = firewall.clone();
    firewall.source_ranges = Some(vec![format!("{}/32", new_ip)]);

    client
        .compute()
        .firewalls()
        .update(firewall_rule_name, &firewall)
        .await
        .map_err(|e| {
            error!("Failed to update firewall rule: {}", e);
            io::Error::new(io::ErrorKind::Other, "Failed to update firewall rule")
        })?;

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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let opt = Opt::from_args();

    info!(
        "Starting dynamic IP updater: firewall_rule_names={:?}",
        opt.firewall_rule_names
    );

    debug!("Initializing in-memory IP address tracker...");
    let mut ip_data = IpAddresses {
        previous_ip: None,
        current_ip: None,
    };

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        debug!("Starting new iteration of the update loop...");
        if let Some(ip) = get_public_ip() {
            if check_ip_changed(&ip, &mut ip_data) {
                for firewall_rule_name in &opt.firewall_rule_names {
                    if let Err(e) = update_firewall_rule(firewall_rule_name, &ip).await {
                        error!("Error updating firewall rule: {}", e);
                    }
                }
            }
        }
        debug!("Sleeping for 3 minutes...");
        sleep(Duration::from_secs(180)); // Sleep for 3 minutes
    });
}