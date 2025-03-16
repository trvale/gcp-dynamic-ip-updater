#!/usr/bin/env python3

# /// script
# dependencies = [
#   "google-api-python-client",
#   "structlog",
#   "requests<3"
# ]
# ///

import requests
from googleapiclient.discovery import build
import structlog
import argparse
import json
import os
import time
import google.auth

# Configure logging
log = structlog.get_logger()

os.environ['GOOGLE_APPLICATION_CREDENTIALS'] = "/secrets/gcp-credentials.json"

def get_public_ip():
    try:
        response = requests.get('https://api.ipify.org?format=json')
        response.raise_for_status()
        ip = response.json()['ip']
        return ip
    except requests.RequestException as e:
        log.error("Failed to fetch public IP address", error=str(e))
        return None

def check_ip_changed(ip):
    file_path = './ip_addresses.json'
    
    if os.path.exists(file_path):
        with open(file_path, 'r') as file:
            data = json.load(file)
    else:
        data = {"previous_ip": None, "current_ip": None}
    
    if data["current_ip"] != ip:
        log.info("IP addres has changed", previous_ip=data["current_ip"], current_ip=ip)
        data["previous_ip"] = data["current_ip"]
        data["current_ip"] = ip
        
        with open(file_path, 'w') as file:
            json.dump(data, file, indent=4)
        return True
    else:
        return False

def update_firewall_rule(firewall_rule_name, new_ip):
    credentials, project = google.auth.default()
    service = build('compute', 'v1', credentials=credentials)
    firewall_rule = service.firewalls().get(project=project, firewall=firewall_rule_name).execute()
    
    firewall_rule['sourceRanges'] = [f'{new_ip}/32']
    
    request = service.firewalls().update(project=project, firewall=firewall_rule_name, body=firewall_rule)
    response = request.execute()
    return response

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Update GCP firewall rule with the current public IP address.')
    parser.add_argument('firewall_rule_names', nargs='+', help='The names of the firewall rules to update')
    args = parser.parse_args()

    firewall_rule_names = args.firewall_rule_names
    log.info("Starting dynamic IP updater", firewall_rule_names=firewall_rule_names, ip=get_public_ip())
    while True:
        ip = get_public_ip()
        if check_ip_changed(ip):
            for firewall_rule_name in firewall_rule_names:
                response = update_firewall_rule(firewall_rule_name, ip)
                log.info("Firewall rule updated", firewall_rule_name=firewall_rule_name, ip=ip, response=response)
        time.sleep(180)  # Sleep for 3 minutes