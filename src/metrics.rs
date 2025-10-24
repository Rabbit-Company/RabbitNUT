use axum::{
	Json, Router,
	extract::State,
	http::{HeaderMap, StatusCode},
	response::{IntoResponse, Response},
	routing::get,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::MetricsConfig;
use crate::ups::UpsStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
	pub ups_name: String,
	pub ups_host: String,
	pub battery_charge_percent: f64,
	pub battery_runtime_seconds: u64,
	pub ups_status: String,
	pub on_battery: bool,
	pub last_update: i64,
	pub on_battery_duration_seconds: Option<u64>,
	pub output_power_watts: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonMetricsResponse {
	pub status: String,
	pub timestamp: i64,
	pub metrics: Metrics,
}

#[derive(Clone)]
pub struct MetricsServer {
	config: MetricsConfig,
	metrics: Arc<RwLock<Option<Metrics>>>,
}

#[derive(Clone)]
struct AppState {
	metrics: Arc<RwLock<Option<Metrics>>>,
	bearer_token: Option<String>,
	format: String,
}

impl MetricsServer {
	pub fn new(config: MetricsConfig) -> Self {
		MetricsServer {
			config,
			metrics: Arc::new(RwLock::new(None)),
		}
	}

	pub async fn update_metrics(
		&self,
		ups_name: String,
		ups_host: String,
		status: UpsStatus,
		on_battery_duration: Option<u64>,
	) {
		let metrics = Metrics {
			ups_name,
			ups_host,
			battery_charge_percent: status.battery_charge,
			battery_runtime_seconds: status.battery_runtime,
			ups_status: status.ups_status,
			on_battery: status.on_battery,
			last_update: chrono::Utc::now().timestamp(),
			on_battery_duration_seconds: on_battery_duration,
			output_power_watts: status.output_power,
		};

		let mut m = self.metrics.write().await;
		*m = Some(metrics);
	}

	pub async fn start(self: Arc<Self>) {
		let port = self.config.port;
		info!("Starting metrics server on port {}", port);

		let state = AppState {
			metrics: self.metrics.clone(),
			bearer_token: self.config.bearer_token.clone(),
			format: self
				.config
				.format
				.clone()
				.unwrap_or_else(|| "openmetrics".to_string()),
		};

		let app = Router::new()
			.route("/metrics", get(handle_metrics))
			.route("/health", get(handle_health))
			.with_state(state);

		let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
			.await
			.expect("Failed to bind to address");

		axum::serve(listener, app)
			.await
			.expect("Failed to start server");
	}
}

async fn handle_health() -> impl IntoResponse {
	(StatusCode::OK, "OK")
}

async fn handle_metrics(
	headers: HeaderMap,
	State(state): State<AppState>,
) -> Result<Response, StatusCode> {
	// Check authorization if token is configured
	if let Some(ref required_token) = state.bearer_token {
		let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());

		match auth_header {
			Some(header) if header == format!("Bearer {}", required_token) => {
				// Authorized
			}
			_ => {
				return Ok((StatusCode::UNAUTHORIZED, "Unauthorized").into_response());
			}
		}
	}

	let metrics_lock = state.metrics.read().await;

	match &*metrics_lock {
		Some(metrics) => {
			if state.format == "json" {
				let response = JsonMetricsResponse {
					status: "ok".to_string(),
					timestamp: chrono::Utc::now().timestamp(),
					metrics: metrics.clone(),
				};
				Ok(Json(response).into_response())
			} else {
				// OpenMetrics format
				let output = format_openmetrics(metrics);
				Ok(
					(
						StatusCode::OK,
						[(
							"content-type",
							"application/openmetrics-text; version=1.0.0; charset=utf-8",
						)],
						output,
					)
						.into_response(),
				)
			}
		}
		None => Ok((StatusCode::SERVICE_UNAVAILABLE, "No metrics available").into_response()),
	}
}

fn format_openmetrics(metrics: &Metrics) -> String {
	let mut output = String::new();

	// Battery charge ratio
	output.push_str("# TYPE ups_battery_charge_ratio gauge\n");
	output.push_str("# UNIT ups_battery_charge_ratio ratio\n");
	output
		.push_str("# HELP ups_battery_charge_ratio Battery charge level as a ratio (0.0 to 1.0).\n");
	output.push_str(&format!(
		"ups_battery_charge_ratio{{ups_name=\"{}\",ups_host=\"{}\"}} {}\n",
		escape_label(&metrics.ups_name),
		escape_label(&metrics.ups_host),
		metrics.battery_charge_percent / 100.0
	));

	// Battery runtime
	output.push_str("# TYPE ups_battery_runtime_seconds gauge\n");
	output.push_str("# UNIT ups_battery_runtime_seconds seconds\n");
	output.push_str("# HELP ups_battery_runtime_seconds Estimated battery runtime in seconds.\n");
	output.push_str(&format!(
		"ups_battery_runtime_seconds{{ups_name=\"{}\",ups_host=\"{}\"}} {}\n",
		escape_label(&metrics.ups_name),
		escape_label(&metrics.ups_host),
		metrics.battery_runtime_seconds
	));

	// On battery status
	output.push_str("# TYPE ups_on_battery gauge\n");
	output.push_str(
		"# HELP ups_on_battery Whether UPS is running on battery (1 = on battery, 0 = on line power).\n",
	);
	output.push_str(&format!(
		"ups_on_battery{{ups_name=\"{}\",ups_host=\"{}\"}} {}\n",
		escape_label(&metrics.ups_name),
		escape_label(&metrics.ups_host),
		if metrics.on_battery { 1 } else { 0 }
	));

	// On battery duration (if applicable)
	if let Some(duration) = metrics.on_battery_duration_seconds {
		output.push_str("# TYPE ups_on_battery_duration_seconds gauge\n");
		output.push_str("# UNIT ups_on_battery_duration_seconds seconds\n");
		output.push_str(
			"# HELP ups_on_battery_duration_seconds Duration in seconds that UPS has been on battery.\n",
		);
		output.push_str(&format!(
			"ups_on_battery_duration_seconds{{ups_name=\"{}\",ups_host=\"{}\"}} {}\n",
			escape_label(&metrics.ups_name),
			escape_label(&metrics.ups_host),
			duration
		));
	}

	// Output power (if available)
	if let Some(power) = metrics.output_power_watts {
		output.push_str("# TYPE ups_output_power_watts gauge\n");
		output.push_str("# UNIT ups_output_power_watts watts\n");
		output.push_str("# HELP ups_output_power_watts Current UPS output power in watts.\n");
		output.push_str(&format!(
			"ups_output_power_watts{{ups_name=\"{}\",ups_host=\"{}\"}} {}\n",
			escape_label(&metrics.ups_name),
			escape_label(&metrics.ups_host),
			power
		));
	}

	// Last update timestamp
	output.push_str("# TYPE ups_last_update_timestamp_seconds gauge\n");
	output.push_str("# UNIT ups_last_update_timestamp_seconds seconds\n");
	output.push_str(
		"# HELP ups_last_update_timestamp_seconds Unix timestamp of last successful UPS status update.\n",
	);
	output.push_str(&format!(
		"ups_last_update_timestamp_seconds{{ups_name=\"{}\",ups_host=\"{}\"}} {}\n",
		escape_label(&metrics.ups_name),
		escape_label(&metrics.ups_host),
		metrics.last_update
	));

	// UPS status info
	output.push_str("# TYPE ups_status_info info\n");
	output.push_str("# HELP ups_status_info UPS status information.\n");
	output.push_str(&format!(
		"ups_status_info{{ups_name=\"{}\",ups_host=\"{}\",status=\"{}\"}} 1\n",
		escape_label(&metrics.ups_name),
		escape_label(&metrics.ups_host),
		escape_label(&metrics.ups_status)
	));

	// OpenMetrics EOF marker
	output.push_str("# EOF\n");

	output
}

// Escape label values according to OpenMetrics specification
fn escape_label(value: &str) -> String {
	value
		.chars()
		.map(|c| match c {
			'"' => "\\\"".to_string(),
			'\\' => "\\\\".to_string(),
			'\n' => "\\n".to_string(),
			c => c.to_string(),
		})
		.collect()
}
