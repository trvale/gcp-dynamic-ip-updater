# GCP Dynamic IP Updater

## Overview

GCP Dynamic IP Updater is a tool designed to automatically update the IP address of a Google Cloud Platform (GCP) resource. This is particularly useful for resources that require a static IP address but are assigned dynamic IPs.

## Features

- Automatically detects changes in IP address
- Updates GCP resource with the new IP address
- Supports multiple resource types

## Prerequisites

- Google Cloud SDK installed
- Properly configured GCP project
- Service account with necessary permissions
- UV installed

## Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/yourusername/gcp-dynamic-ip-updater.git
    ```
2. Navigate to the project directory:
    ```sh
    cd gcp-dynamic-ip-updater
    ```
3. Install UV:
    ```sh
    curl -LsSf https://astral.sh/uv/install.sh | sh
    ```

## Configuration

1. Login to GCP:
    ```sh
    gcloud auth application-default login
    ```
2. Move or copy login credentials to secrets folder:
    ```sh
    mkdir -p ./secrets
    cp /Users/jtravaille/.config/gcloud/application_default_credentials.json ./secrets/gcp-credentials.json
    ```

## Usage

Run the updater script:
```sh
uv run gcp-dynamic-ip-updater.py <1st-firewall-rule-name> <nth-firewall-rule-name>
```

Or use docker:
```sh
./build-docker.sh
docker run -it --rm gcp-dynamic-ip-updater
```

## Contributing

Contributions are welcome! Please submit a pull request or open an issue to discuss your ideas.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
