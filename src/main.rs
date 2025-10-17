mod config;
mod logging;
mod monitor;
mod ups;

use log::info;
use std::env;

use crate::config::Config;
use crate::logging::setup_logging;
use crate::monitor::UpsMonitor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args: Vec<String> = env::args().collect();

	if args.iter().any(|a| a == "--version" || a == "-v") {
		println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
		return Ok(());
	}

	let config_path = env::args()
		.nth(1)
		.unwrap_or_else(|| "config.toml".to_string());

	let config = Config::from_file(&config_path)?;

	setup_logging(&config.logging)?;

	info!("UPS Monitor started with config: {}", config_path);

	let mut monitor: UpsMonitor = UpsMonitor::new(config);
	monitor.run();

	Ok(())
}
