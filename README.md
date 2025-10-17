# RabbitNUT

This Rust program serves as a comprehensive UPS monitoring tool using the Network UPS Tools (NUT) protocol. It monitors UPS status, battery levels, and runtime, with configurable automatic shutdown capabilities for system protection.

## Features

- **UPS Monitoring**: Real-time monitoring of UPS status, battery charge, and runtime
- **Automatic Shutdown**: Configurable shutdown conditions to protect your system
- **NUT Protocol Support**: Compatible with Network UPS Tools servers
- **Flexible Configuration**: Easy setup via config.toml file
- **Comprehensive Logging**: Detailed logging with configurable levels

## Configuration

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
```

## Configuration Sections

### UPS Connection

- **host**: IP address or hostname of your NUT server
- **name**: Name of the UPS as configured in NUT
- **port**: NUT server port (default: 3493)
- **username**: Authentication username
- **password**: Authentication password

### Monitoring

- **poll_interval**: How frequently to check UPS status (in seconds)

### Shutdown Conditions

RabbitNUT will trigger a shutdown if ANY of these conditions are met:

- **on_battery_seconds**: UPS has been running on battery for specified time
- **battery_percent_threshold**: Battery charge drops below specified percentage
- **runtime_threshold**: Estimated runtime falls below threshold (in seconds)

### Logging

- **log_file**: Path where log files will be stored
- **log_level**: Verbosity of logging (trace, debug, info, warn, error)

## Installation

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

## Daemonizing (using systemd)

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

## Upgrade

```bash
# Stop service
systemctl stop rabbitnut

# Download Pulse Monitor
wget https://github.com/Rabbit-Company/RabbitNUT/releases/latest/download/rabbitnut-$(uname -m)-gnu
sudo chmod 755 rabbitnut-$(uname -m)-gnu
sudo mv rabbitnut-$(uname -m)-gnu /usr/local/bin/rabbitnut

# Start service
systemctl start rabbitnut
```
