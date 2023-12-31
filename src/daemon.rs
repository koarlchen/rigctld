// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::Read;
use std::process::Stdio;

use tokio::io;
use tokio::process::{Child, Command};

/// Representation of hamlib's `rigctld`.
pub struct Rigctld {
    daemon: Option<Child>,
}

/// Rigctld implementation.
impl Rigctld {
    /// Create new instance of `Rigctld`.
    pub fn new(child: Child) -> Self {
        Self {
            daemon: Some(child),
        }
    }

    /// Kill a running instance of `Rigctld`.
    pub async fn kill(&mut self) -> Result<(), io::Error> {
        let res = if let Some(d) = self.daemon.as_mut() {
            d.kill().await
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Daemon not started",
            ))
        };

        self.daemon = None;

        res
    }

    /// Check if `Rigctld` is running.
    pub fn is_running(&mut self) -> Result<bool, io::Error> {
        if let Some(d) = self.daemon.as_mut() {
            match d.try_wait()? {
                Some(_) => {
                    self.daemon = None;
                    Ok(false)
                }
                None => Ok(true),
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Daemon not started",
            ))
        }
    }
}

/// Representation of `rigctld` commandline parameters.
#[derive(Debug)]
pub struct Daemon {
    program: String,
    host: String,
    port: u16,
    model: u32,
    rig_file: Option<String>,
    serial_speed: Option<u32>,
    civ_address: Option<u16>,
}

impl Default for Daemon {
    /// Get default `rigctld` configuration.
    /// If spawned, `rigctld` opens the socket on `127.0.0.1:4532` and will use the dummy device model.
    fn default() -> Self {
        Daemon {
            program: "rigctld".into(),
            host: "127.0.0.1".into(),
            port: 4532,
            model: 1,
            rig_file: None,
            serial_speed: None,
            civ_address: None,
        }
    }
}

/// Deamon implementation.
impl Daemon {
    /// Spawn new instance of `Rigctld`.
    pub async fn spawn(&self) -> Result<Rigctld, io::Error> {
        let mut binding = Command::new(self.program.clone());
        binding.kill_on_drop(true);
        let cmd = binding
            .args(["-T", &self.host])
            .args(["-t", &self.port.to_string()])
            .args(["-m", &self.model.to_string()]);

        if let Some(rig) = self.rig_file.as_ref() {
            cmd.args(["-r", rig]);
        }
        if let Some(speed) = self.serial_speed.as_ref() {
            cmd.args(["-s", &speed.to_string()]);
        }
        if let Some(civ) = self.civ_address.as_ref() {
            cmd.args(["-c", &civ.to_string()]);
        }

        let daemon = Rigctld::new(cmd.spawn()?);

        Ok(daemon)
    }

    /// Get version of `rigctld`.
    /// Therefore spawns and returns the output of the command `rigctld --version`.
    pub async fn get_version(&self) -> Result<String, io::Error> {
        let child = Command::new(self.program.clone())
            .stdout(Stdio::piped())
            .arg("--version")
            .spawn()?
            .wait_with_output()
            .await?;

        let mut version: String = String::new();
        child.stdout.as_slice().read_to_string(&mut version)?;
        version = String::from(version.trim_end());

        Ok(version)
    }

    /// Sets the name of the `rigctld` program.
    /// The name may be prefixed with the path to `rigctld` if it is not present within the systems `PATH`.
    pub fn set_program(mut self, app: String) -> Daemon {
        self.program = app;
        self
    }

    /// Set the host or rather ip address to open the listening socket on.
    pub fn set_host(mut self, host: String) -> Daemon {
        self.host = host;
        self
    }

    /// Get host for communication to daemon.
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Set the port to open the listening socket on.
    pub fn set_port(mut self, port: u16) -> Daemon {
        self.port = port;
        self
    }

    /// Get port for communication to daemon.
    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// Set the device model. See `rigctld -l` for supported models.
    pub fn set_model(mut self, model: u32) -> Daemon {
        self.model = model;
        self
    }

    /// Set the rigs device file, e.g. `/dev/ttyUSB0`.
    pub fn set_rig_file(mut self, file: String) -> Daemon {
        self.rig_file = Some(file);
        self
    }

    /// Set the rigs serial speed, e.g. 19200.
    pub fn set_serial_speed(mut self, speed: u32) -> Daemon {
        self.serial_speed = Some(speed);
        self
    }

    /// Set the rigs CIV address, e.g. 0x76.
    pub fn set_civ_address(mut self, addr: u16) -> Daemon {
        self.civ_address = Some(addr);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    macro_rules! tokio {
        ($e:expr) => {
            Runtime::new().unwrap().block_on(async { $e })
        };
    }

    #[test]
    fn rigctld_exists() {
        tokio!({
            Daemon::default().spawn().await.unwrap();
        })
    }

    #[test]
    fn rigctld_version() {
        tokio!({
            Daemon::default().get_version().await.unwrap();
        })
    }

    #[test]
    fn daemon_lifecycle() {
        tokio!({
            let mut d = Daemon::default().spawn().await.unwrap();
            assert_eq!(d.is_running().unwrap(), true);
            d.kill().await.unwrap();
        })
    }

    #[test]
    fn daemon_kill_twice() {
        tokio!({
            let mut d = Daemon::default().spawn().await.unwrap();
            d.kill().await.unwrap();
            assert_eq!(d.kill().await.is_err(), true);
        })
    }
}
