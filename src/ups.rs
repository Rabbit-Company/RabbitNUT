use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub struct UpsStatus {
	pub battery_charge: f64,
	pub battery_runtime: u64,
	pub ups_status: String,
	pub on_battery: bool,
	pub output_power: Option<f64>,
}

impl fmt::Display for UpsStatus {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"Charge: {}%, Runtime: {}s, Status: {}, On Battery: {}",
			self.battery_charge, self.battery_runtime, self.ups_status, self.on_battery
		)
	}
}

pub struct UpsClient {
	host: String,
	port: u16,
	name: String,
	username: Option<String>,
	password: Option<String>,
}

impl UpsClient {
	pub fn new(
		host: String,
		port: u16,
		name: String,
		username: Option<String>,
		password: Option<String>,
	) -> Self {
		UpsClient {
			host,
			port,
			name,
			username,
			password,
		}
	}

	fn connect(&self) -> Result<TcpStream, Box<dyn std::error::Error>> {
		let addr = format!("{}:{}", self.host, self.port);
		let mut stream = TcpStream::connect(addr)?;

		if self.username.is_some() && self.password.is_some() {
			self.authenticate(&mut stream)?;
		}

		Ok(stream)
	}

	fn authenticate(&self, stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
		let username = self.username.as_ref().unwrap();
		let password = self.password.as_ref().unwrap();

		let user_cmd = format!("USERNAME {}\n", username);
		stream.write_all(user_cmd.as_bytes())?;

		let mut reader = BufReader::new(stream.try_clone()?);
		let mut response = String::new();
		reader.read_line(&mut response)?;

		if !response.to_uppercase().contains("OK") {
			return Err(format!("Authentication failed at USERNAME: {}", response).into());
		}

		let pass_cmd = format!("PASSWORD {}\n", password);
		stream.write_all(pass_cmd.as_bytes())?;

		response.clear();
		reader.read_line(&mut response)?;

		if !response.to_uppercase().contains("OK") {
			return Err(format!("Authentication failed at PASSWORD: {}", response).into());
		}

		Ok(())
	}

	fn get_var(
		&self,
		stream: &mut TcpStream,
		var_name: &str,
	) -> Result<String, Box<dyn std::error::Error>> {
		let command = format!("GET VAR {} {}\n", self.name, var_name);
		stream.write_all(command.as_bytes())?;

		let mut reader = BufReader::new(stream.try_clone()?);
		let mut response = String::new();
		reader.read_line(&mut response)?;

		let parts: Vec<&str> = response.trim().split_whitespace().collect();
		if parts.len() >= 4 && parts[0] == "VAR" {
			let value = parts[3..].join(" ").trim_matches('"').to_string();
			Ok(value)
		} else if response.contains("ERR") {
			Err(format!("UPS error response: {}", response).into())
		} else {
			Err(format!("Invalid response: {}", response).into())
		}
	}

	pub fn get_status(&self) -> Result<UpsStatus, Box<dyn std::error::Error>> {
		let mut stream = self.connect()?;

		let battery_charge = self
			.get_var(&mut stream, "battery.charge")?
			.parse::<f64>()
			.unwrap_or(0.0);

		let battery_runtime = self
			.get_var(&mut stream, "battery.runtime")?
			.parse::<u64>()
			.unwrap_or(0);

		let ups_status = self.get_var(&mut stream, "ups.status")?;
		let on_battery = ups_status.contains("OB") || ups_status.contains("DISCHRG");

		let output_power = match self.get_var(&mut stream, "output.power") {
			Ok(v) => v.parse::<f64>().ok(),
			Err(_) => None,
		};

		Ok(UpsStatus {
			battery_charge,
			battery_runtime,
			ups_status,
			on_battery,
			output_power,
		})
	}

	pub fn list_vars(&self) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
		let mut stream = self.connect()?;
		let command = format!("LIST VAR {}\n", self.name);
		stream.write_all(command.as_bytes())?;

		let reader = BufReader::new(stream.try_clone()?);
		let mut vars = Vec::new();

		for line in reader.lines() {
			let line = line?;
			if line.starts_with("VAR") {
				let parts: Vec<&str> = line.split_whitespace().collect();
				if parts.len() >= 4 {
					let var_name = parts[2].to_string();
					let value = parts[3..].join(" ").trim_matches('"').to_string();
					vars.push((var_name, value));
				}
			} else if line.starts_with("END LIST") {
				break;
			}
		}

		Ok(vars)
	}
}
