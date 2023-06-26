// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::process::{Child, Command};

/// Representation of `rigctld`.
#[derive(Debug)]
pub struct Daemon {
    program: String,
    host: String,
    port: u16,
    model: u32,
    rig_file: Option<String>,
    serial_speed: Option<u32>,
    civ_address: Option<u16>,
    daemon: Option<Child>,
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
            daemon: None,
        }
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        _ = self.kill();
    }
}

/// Deamon implementation.
impl Daemon {
    /// Spawn new instance of `rigctld`.
    pub fn spawn(&mut self) -> Result<(), io::Error> {
        if self.daemon.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Daemon already started",
            ));
        }

        let mut binding = Command::new(self.program.clone());
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

        self.daemon = Some(cmd.spawn()?);

        Ok(())
    }

    /// Kill a running instance of `rigctld`.
    pub fn kill(&mut self) -> Result<(), io::Error> {
        let res = if let Some(d) = self.daemon.as_mut() {
            d.kill()
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Daemon not started",
            ))
        };

        self.daemon = None;

        res
    }

    /// Check if `rigctld` is still running.
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

    /// Sets the name of the `rigctld` program.
    /// The name may be prefixed with the path to `rigctld` if it is not present within the `PATH` variable.
    pub fn set_program(mut self, app: String) -> Daemon {
        self.program = app;
        self
    }

    /// Set the host or rather ip address to open the listening socket on.
    pub fn set_host(mut self, host: String) -> Daemon {
        self.host = host;
        self
    }

    /// Set the port to open the listening socket on.
    pub fn set_port(mut self, port: u16) -> Daemon {
        self.port = port;
        self
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

    #[test]
    fn rigctld_exists() {
        Daemon::default().spawn().unwrap();
    }
}
