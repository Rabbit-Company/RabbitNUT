use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
	pub ups: UpsConfig,
	pub monitoring: MonitoringConfig,
	pub shutdown: ShutdownConfig,
	pub logging: LoggingConfig,
	pub metrics: Option<MetricsConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpsConfig {
	pub host: String,
	pub name: String,
	pub port: u16,
	pub username: Option<String>,
	pub password: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MonitoringConfig {
	pub poll_interval: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ShutdownConfig {
	pub enabled: bool,
	pub on_battery_seconds: u64,
	pub battery_percent_threshold: f64,
	pub runtime_threshold: u64,
	pub shutdown_command: String,
	pub shutdown_grace_period: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
	pub log_file: Option<String>,
	pub log_level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
	pub enabled: bool,
	pub port: u16,
	pub bearer_token: Option<String>,
	pub format: Option<String>,
}

impl Config {
	pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
		let config_str = fs::read_to_string(path)?;
		let config: Config = toml::from_str(&config_str)?;
		Ok(config)
	}
}

impl Default for Config {
	fn default() -> Self {
		Config {
			ups: UpsConfig {
				host: "localhost".to_string(),
				name: "ups".to_string(),
				port: 3493,
				username: None,
				password: None,
			},
			monitoring: MonitoringConfig { poll_interval: 5 },
			shutdown: ShutdownConfig {
				enabled: false,
				on_battery_seconds: 300,
				battery_percent_threshold: 20.0,
				runtime_threshold: 180,
				shutdown_command: "/sbin/shutdown -h +0".to_string(),
				shutdown_grace_period: 30,
			},
			logging: LoggingConfig {
				log_file: None,
				log_level: "info".to_string(),
			},
			metrics: Some(MetricsConfig {
				enabled: false,
				port: 8089,
				bearer_token: None,
				format: Some("openmetrics".to_string()),
			}),
		}
	}
}
