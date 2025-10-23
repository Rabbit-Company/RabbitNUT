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
# UPS Monitor Configuration

[ups]
# UPS connection details
host = "10.0.31.1"
name = "ups1"
port = 3493  # Default NUT port
username = "admin"
password = "Password123"

[monitoring]
# Polling interval in seconds
poll_interval = 5

[shutdown]
# Enable automatic shutdown
enabled = true

# Shutdown conditions (any condition met will trigger shutdown)
# Time in seconds after switching to battery
on_battery_seconds = 300  # 5 minutes

# Battery charge percentage threshold
battery_percent_threshold = 20

# Runtime threshold in seconds
runtime_threshold = 180  # 3 minutes

# Shutdown command (adjust for your system)
shutdown_command = "/sbin/shutdown -h +0"

# Grace period before executing shutdown (seconds)
shutdown_grace_period = 30

[logging]
# Log file path
log_file = "/var/log/rabbitnut.log"
# Log level: trace, debug, info, warn, error
log_level = "info"

[metrics]
# Enable metrics API endpoint
enabled = true

# Port to listen on for metrics endpoint
port = 8089

# Optional bearer token for authentication
# If not set, no authentication is required
bearer_token = "secure-monitoring-token-123"

# Metrics format: "openmetrics" or "json"
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
