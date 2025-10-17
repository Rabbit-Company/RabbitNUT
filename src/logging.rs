use chrono::Local;
use log::LevelFilter;
use std::fs::OpenOptions;

use crate::config::LoggingConfig;

pub fn setup_logging(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
	let log_level = parse_log_level(&config.log_level);

	let _ = OpenOptions::new()
		.create(true)
		.append(true)
		.open(&config.log_file)?;

	fern::Dispatch::new()
		.level(log_level)
		.level_for("rabbitnut", log_level)
		.chain(create_stdout_logger(log_level))
		.chain(create_file_logger(&config.log_file, log_level)?)
		.apply()?;

	Ok(())
}

fn parse_log_level(level: &str) -> LevelFilter {
	match level.to_lowercase().as_str() {
		"trace" => LevelFilter::Trace,
		"debug" => LevelFilter::Debug,
		"info" => LevelFilter::Info,
		"warn" | "warning" => LevelFilter::Warn,
		"error" => LevelFilter::Error,
		"off" => LevelFilter::Off,
		_ => {
			eprintln!("Unknown log level '{}', defaulting to 'info'", level);
			LevelFilter::Info
		}
	}
}

fn create_stdout_logger(level: LevelFilter) -> fern::Dispatch {
	fern::Dispatch::new()
		.format(|out, message, record| {
			use colored::*;

			let level_string = match record.level() {
				log::Level::Error => "ERROR".red().bold(),
				log::Level::Warn => "WARN ".yellow().bold(),
				log::Level::Info => "INFO ".green(),
				log::Level::Debug => "DEBUG".blue(),
				log::Level::Trace => "TRACE".purple(),
			};

			out.finish(format_args!(
				"{} {} {}",
				Local::now().format("%Y-%m-%d %H:%M:%S"),
				level_string,
				message
			))
		})
		.level(level)
		.chain(std::io::stdout())
}

fn create_file_logger(
	path: &str,
	level: LevelFilter,
) -> Result<fern::Dispatch, Box<dyn std::error::Error>> {
	Ok(
		fern::Dispatch::new()
			.format(|out, message, record| {
				out.finish(format_args!(
					"{} [{}] {}",
					chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
					record.level(),
					message
				))
			})
			.level(level)
			.chain(fern::log_file(path)?),
	)
}
