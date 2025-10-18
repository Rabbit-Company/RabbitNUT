use log::{debug, error, info, warn};
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

use crate::config::Config;
use crate::metrics::MetricsServer;
use crate::ups::{UpsClient, UpsStatus};

pub struct UpsMonitor {
	config: Config,
	client: UpsClient,
	state: MonitorState,
	metrics_server: Option<Arc<MetricsServer>>,
	runtime: Option<Runtime>,
}

struct MonitorState {
	on_battery_since: Option<Instant>,
	shutdown_scheduled: bool,
}

impl UpsMonitor {
	pub fn new(config: Config) -> Self {
		let client = UpsClient::new(
			config.ups.host.clone(),
			config.ups.port,
			config.ups.name.clone(),
			config.ups.username.clone(),
			config.ups.password.clone(),
		);

		// Initialize metrics server if enabled
		let (metrics_server, runtime) = if let Some(ref metrics_config) = config.metrics {
			if metrics_config.enabled {
				let runtime = Runtime::new().expect("Failed to create Tokio runtime");
				let server = Arc::new(MetricsServer::new(metrics_config.clone()));
				(Some(server), Some(runtime))
			} else {
				(None, None)
			}
		} else {
			(None, None)
		};

		UpsMonitor {
			config,
			client,
			state: MonitorState {
				on_battery_since: None,
				shutdown_scheduled: false,
			},
			metrics_server,
			runtime,
		}
	}

	pub fn run(&mut self) {
		info!(
			"Starting UPS monitor for {}@{}",
			self.config.ups.name, self.config.ups.host
		);

		// Start metrics server if enabled
		if let Some(ref server) = self.metrics_server {
			if let Some(ref runtime) = self.runtime {
				let server_clone = server.clone();
				runtime.spawn(async move {
					server_clone.start().await;
				});
				info!("Metrics server started");
			}
		}

		self.print_ups_info();

		loop {
			if let Err(e) = self.monitor_cycle() {
				error!("Monitor cycle error: {}", e);
			}

			if self.state.shutdown_scheduled {
				break;
			}

			thread::sleep(Duration::from_secs(self.config.monitoring.poll_interval));
		}
	}

	fn print_ups_info(&self) {
		info!("Attempting to connect to UPS and retrieve variables...");

		match self.client.list_vars() {
			Ok(vars) => {
				info!("Connected successfully");
				debug!("UPS variables:");
				for (name, value) in vars {
					debug!("  {}: {}", name, value);
				}
			}
			Err(e) => {
				warn!("Failed to list UPS variables: {}", e);
			}
		}
	}

	fn monitor_cycle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		let status = self.client.get_status()?;

		debug!("UPS Status: {}", status);

		// Update metrics if server is enabled
		if let Some(ref server) = self.metrics_server {
			if let Some(ref runtime) = self.runtime {
				let on_battery_duration = self
					.state
					.on_battery_since
					.map(|since| since.elapsed().as_secs());

				let server_clone = server.clone();
				let ups_name = self.config.ups.name.clone();
				let ups_host = self.config.ups.host.clone();
				let status_clone = status.clone();

				runtime.spawn(async move {
					server_clone
						.update_metrics(ups_name, ups_host, status_clone, on_battery_duration)
						.await;
				});
			}
		}

		self.update_battery_state(&status);

		if self.should_shutdown(&status) {
			self.execute_shutdown();
		}

		Ok(())
	}

	fn update_battery_state(&mut self, status: &UpsStatus) {
		if status.on_battery {
			if self.state.on_battery_since.is_none() {
				self.state.on_battery_since = Some(Instant::now());
				warn!("⚠️  UPS switched to battery power!");
				self.log_battery_status(status);
			}
		} else if self.state.on_battery_since.is_some() {
			info!("✓ UPS back on line power");
			self.state.on_battery_since = None;
		}
	}

	fn log_battery_status(&self, status: &UpsStatus) {
		info!(
			"Battery status - Charge: {}%, Runtime: {} minutes",
			status.battery_charge,
			status.battery_runtime / 60
		);

		if self.config.shutdown.enabled {
			info!("Shutdown thresholds:");
			info!(
				"  - After {} seconds on battery",
				self.config.shutdown.on_battery_seconds
			);
			info!(
				"  - Below {}% charge",
				self.config.shutdown.battery_percent_threshold
			);
			info!(
				"  - Below {} seconds runtime",
				self.config.shutdown.runtime_threshold
			);
		}
	}

	fn should_shutdown(&mut self, status: &UpsStatus) -> bool {
		if !self.config.shutdown.enabled || self.state.shutdown_scheduled {
			return false;
		}

		if !status.on_battery {
			return false;
		}

		// Check time on battery
		if let Some(since) = self.state.on_battery_since {
			let elapsed = since.elapsed().as_secs();
			if elapsed >= self.config.shutdown.on_battery_seconds {
				error!(
					"🔴 UPS on battery for {} seconds (threshold: {}), triggering shutdown",
					elapsed, self.config.shutdown.on_battery_seconds
				);
				return true;
			}

			// Log remaining time periodically
			let remaining = self.config.shutdown.on_battery_seconds - elapsed;
			if remaining % 60 == 0 || remaining <= 30 {
				warn!("Time until shutdown: {} seconds", remaining);
			}
		}

		// Check battery charge threshold
		if status.battery_charge <= self.config.shutdown.battery_percent_threshold {
			error!(
				"🔴 Battery charge {}% below threshold {}%, triggering shutdown",
				status.battery_charge, self.config.shutdown.battery_percent_threshold
			);
			return true;
		}

		// Check runtime threshold
		if status.battery_runtime <= self.config.shutdown.runtime_threshold {
			error!(
				"🔴 Battery runtime {} seconds below threshold {}, triggering shutdown",
				status.battery_runtime, self.config.shutdown.runtime_threshold
			);
			return true;
		}

		false
	}

	fn execute_shutdown(&mut self) {
		if self.state.shutdown_scheduled {
			return;
		}

		self.state.shutdown_scheduled = true;

		error!(
			"🚨 INITIATING SYSTEM SHUTDOWN IN {} SECONDS! 🚨",
			self.config.shutdown.shutdown_grace_period
		);

		// Log countdown
		for i in (1..=self.config.shutdown.shutdown_grace_period).rev() {
			if i <= 10 || i % 10 == 0 {
				warn!("Shutdown in {} seconds...", i);
			}
			thread::sleep(Duration::from_secs(1));
		}

		// Parse and execute shutdown command
		let parts: Vec<&str> = self
			.config
			.shutdown
			.shutdown_command
			.split_whitespace()
			.collect();

		if parts.is_empty() {
			error!("Shutdown command is empty!");
			return;
		}

		info!(
			"Executing shutdown command: {}",
			self.config.shutdown.shutdown_command
		);

		match Command::new(parts[0]).args(&parts[1..]).output() {
			Ok(output) => {
				if output.status.success() {
					info!("Shutdown command executed successfully");
				} else {
					error!(
						"Shutdown command failed: {:?}",
						String::from_utf8_lossy(&output.stderr)
					);
				}
			}
			Err(e) => {
				error!("Failed to execute shutdown command: {}", e);
				error!(
					"Please ensure the command '{}' is valid and accessible",
					parts[0]
				);
			}
		}
	}
}
