# 🐇 RabbitNUT

RabbitNUT is a Rust-based UPS monitoring tool that communicates with Network UPS Tools (NUT) servers.

It provides real-time UPS monitoring, safe system shutdown during power events, and optional metrics export for external monitoring systems.

## 🚀 Features

- ⚡ **UPS Monitoring** — Continuously tracks UPS status, battery charge, and estimated runtime
- 🔋 **Automatic Shutdown** — Graceful shutdown when configurable conditions are met
- 🌐 **NUT Protocol Support** — Works with any Network UPS Tools (NUT)–compatible UPS
- ⚙️ **Flexible Configuration** — Simple, TOML-based configuration file
- 🧾 **Comprehensive Logging** — Adjustable log levels for detailed diagnostics
- 📊 **Metrics Endpoint** — Optional metrics in JSON or OpenMetrics format for Prometheus and similar tools

## ⚙️ Configuration

Before running RabbitNUT, create a `config.toml` file with your UPS and monitoring settings:

```toml
# ======================================
# RabbitNUT - UPS Monitor Configuration
# ======================================

[ups]
# Network UPS Tools (NUT) server connection settings
# These parameters define how to connect to your NUT server

# IP address or hostname of the NUT server
# Examples: "192.168.1.100", "ups.local", "localhost"
host = "10.0.31.1"

# UPS name as configured on the NUT server
# This must match the UPS name defined in the NUT server's ups.conf file
# Use 'upsc -l' on the NUT server to list available UPS names
# Common examples: "ups", "ups1", "apc1500", "eaton5px"
name = "ups"

# NUT server port number
# Default: 3493 (standard NUT port)
port = 3493

# NUT server authentication credentials (optional)
# Uncomment and set these only if your NUT server requires authentication
# These must match a user configured in your NUT server's upsd.users file
#username = "admin"
#password = "Password123"

[monitoring]
# How often to poll the UPS for status updates (in seconds)
# Lower values = more responsive but higher network/CPU usage
# Recommended: 5-30 seconds for most deployments
poll_interval = 5

[shutdown]
# Automatic shutdown configuration
# Controls when and how the system shuts down during power events

# Master switch for automatic shutdown functionality
# Set to false to only monitor without taking action
enabled = true

# === Shutdown Triggers ===
# The system will initiate shutdown when ANY of these conditions are met:

# 1. Time on battery power (in seconds)
# Triggers shutdown after running on battery for this duration
# Example: 300 = shut down after 5 minutes on battery
on_battery_seconds = 300

# 2. Minimum battery charge level (percentage)
# Triggers shutdown when battery drops below this level
# Range: 0-100
# Recommended value depends on your specific setup:
#   - UPS capacity vs load (higher load = set higher threshold)
#   - Battery age and health (older batteries = set higher threshold)
#   - Critical system requirements (longer graceful shutdown = higher threshold)
battery_percent_threshold = 20

# 3. Estimated runtime remaining (in seconds)
# Triggers shutdown when UPS reports less runtime available
# Example: 180 = shut down with 3 minutes runtime left
runtime_threshold = 180

# === Shutdown Execution ===

# System command to execute for shutdown
# Linux examples:
#   - Immediate: "/sbin/shutdown -h now"
#   - With delay: "/sbin/shutdown -h +1" (1 minute warning)
#   - Power off: "/sbin/poweroff"
# Windows example: "shutdown /s /t 0"
# macOS example: "sudo shutdown -h now"
shutdown_command = "/sbin/shutdown -h +0"

# Delay before executing shutdown command (in seconds)
# Gives time to save work or cancel if power returns
# During this period, shutdown can be cancelled if conditions improve
shutdown_grace_period = 30

[logging]
# Application logging configuration

# File path for log output (optional)
# Uncomment to enable file logging
# Ensure the directory exists and is writable by the service user
# Common locations:
#   - Linux: "/var/log/rabbitnut.log"
#   - Windows: "C:\\ProgramData\\rabbitnut\\rabbitnut.log"
# If not specified, logs will only be written to stdout/stderr
#log_file = "/var/log/rabbitnut.log"

# Logging verbosity level
# Options (from least to most verbose):
#   - "error": Only errors
#   - "warn":  Errors and warnings
#   - "info":  Normal operation messages (recommended)
#   - "debug": Detailed operational information
#   - "trace": Very detailed debugging information
log_level = "info"

[metrics]
# Metrics API endpoint configuration
# Exposes UPS status data for monitoring systems (Prometheus, Grafana, etc.)

# Enable/disable the metrics HTTP endpoint
enabled = false

# TCP port for the metrics HTTP server
# Ensure this port is not already in use
port = 8089

# Optional security token for accessing metrics
# When set, requests must include header: "Authorization: Bearer <token>"
# Comment out for no authentication (not recommended for production)
# Generate a secure token with: openssl rand -hex 32
bearer_token = "secure-monitoring-token-123"

# Output format for metrics data
# Options:
#   - "openmetrics": Prometheus/OpenMetrics text format (recommended)
#   - "json": JSON format for custom integrations
format = "openmetrics"
```

## 📘 Configuration Sections

### 🔌 UPS Connection

- `host`: IP or hostname of NUT server
- `name`: UPS name as configured in NUT
- `port`: NUT server port (default: 3493)
- `username`: NUT Authentication username
- `password`: NUT Authentication password

### ⏱️ Monitoring

- `poll_interval`: How often to query UPS status (seconds)

### ⚠️ Shutdown Behavior

RabbitNUT triggers a shutdown when **any** of the following are true:

- UPS has been on battery longer than `on_battery_seconds`
- Battery charge falls below `battery_percent_threshold`
- Estimated runtime is under `runtime_threshold`

### 🪵 Logging

- `log_file`: Path to log file
- `log_level`: Verbosity of logging (trace, debug, info, warn, error)

### 📈 Metrics

- `enabled`: Enables or disables metrics endpoint
- `port`: Port to listen for metrics requests
- `bearer_token`: Optional token for API protection
- `format`: Output format (openmetrics or json)

## 🧩 Installation

```bash
# Download the binary
wget https://github.com/Rabbit-Company/RabbitNUT/releases/latest/download/rabbitnut-$(uname -m)-gnu
# Set file permissions
sudo chmod 755 rabbitnut-$(uname -m)-gnu
# Place the binary to `/usr/local/bin`
sudo mv rabbitnut-$(uname -m)-gnu /usr/local/bin/rabbitnut
# Start the monitor and don't forget to change the path to your config.toml file
rabbitnut /etc/rabbitnut/config.toml
```

## 🧠 Daemonizing (using systemd)

Running RabbitNUT in the background is a simple task, just make sure that it runs without errors before doing this. Place the contents below in a file called `rabbitnut.service` in the `/etc/systemd/system/` directory.

```service
[Unit]
Description=RabbitNUT
After=network.target

[Service]
Type=simple
User=root
ExecStart=rabbitnut /etc/rabbitnut/config.toml
TimeoutStartSec=0
TimeoutStopSec=2
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
```

Then, run the commands below to reload systemd and start RabbitNUT.

```bash
systemctl enable --now rabbitnut
```

## 🔄 Upgrade

```bash
# Download latest version of RabbitNut
wget https://github.com/Rabbit-Company/RabbitNUT/releases/latest/download/rabbitnut-$(uname -m)-gnu
sudo chmod 755 rabbitnut-$(uname -m)-gnu
sudo mv rabbitnut-$(uname -m)-gnu /usr/local/bin/rabbitnut

# Restart service
systemctl restart rabbitnut
```
