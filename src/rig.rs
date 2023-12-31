// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::time;

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    USB,
    LSB,
    CW,
    CWR,
    RTTY,
    RTTYR,
    AM,
    FM,
    WFM,
    AMS,
    PKTLSB,
    PKTUSB,
    PKTFM,
    ECSSUSB,
    ECSSLSB,
    FAX,
    SAM,
    SAL,
    SAH,
    DSB,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::USB => write!(f, "USB"),
            Mode::LSB => write!(f, "LSB"),
            Mode::CW => write!(f, "CW"),
            Mode::CWR => write!(f, "CWR"),
            Mode::RTTY => write!(f, "RTTY"),
            Mode::RTTYR => write!(f, "RTTYR"),
            Mode::AM => write!(f, "AM"),
            Mode::FM => write!(f, "FM"),
            Mode::WFM => write!(f, "WFM"),
            Mode::AMS => write!(f, "AMS"),
            Mode::PKTLSB => write!(f, "PKTLSB"),
            Mode::PKTUSB => write!(f, "PKTUSB"),
            Mode::PKTFM => write!(f, "PKTFM"),
            Mode::ECSSUSB => write!(f, "ECSSUSB"),
            Mode::ECSSLSB => write!(f, "ECSSLSB"),
            Mode::FAX => write!(f, "FAX"),
            Mode::SAM => write!(f, "SAM"),
            Mode::SAL => write!(f, "SAL"),
            Mode::SAH => write!(f, "SAH"),
            Mode::DSB => write!(f, "DSB"),
        }
    }
}

impl FromStr for Mode {
    type Err = RigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USB" => Ok(Mode::USB),
            "LSB" => Ok(Mode::LSB),
            "CW" => Ok(Mode::CW),
            "CWR" => Ok(Mode::CWR),
            "RTTY" => Ok(Mode::RTTY),
            "RTTYR" => Ok(Mode::RTTYR),
            "AM" => Ok(Mode::AM),
            "FM" => Ok(Mode::FM),
            "WFM" => Ok(Mode::WFM),
            "AMS" => Ok(Mode::AMS),
            "PKTLSB" => Ok(Mode::PKTLSB),
            "PKTUSB" => Ok(Mode::PKTUSB),
            "PKTFM" => Ok(Mode::PKTFM),
            "ECSSUSB" => Ok(Mode::ECSSUSB),
            "ECSSLSB" => Ok(Mode::ECSSLSB),
            "FAX" => Ok(Mode::FAX),
            "SAM" => Ok(Mode::SAM),
            "SAL" => Ok(Mode::SAL),
            "SAH" => Ok(Mode::SAH),
            "DSB" => Ok(Mode::DSB),
            _ => Err(RigError::InternalError),
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RigError {
    /// Failed to connect to `rigctld`
    #[error("Failed to connect")]
    ConnectionError,

    /// Timeout in communication to `rigctld`
    #[error("Timeout in communication")]
    CommunicationTimeout,

    /// Lost connection to to `rigctld`
    #[error("Connection lost")]
    ConnectionLost,

    /// Already connected to `rigctld`
    #[error("Already connected")]
    AlreadyConnected,

    /// Internal error
    #[error("Internal error")]
    InternalError,
}

/// Representation of a connection to `rigctld`.
pub struct Rig {
    host: String,
    port: u16,
    reader: Option<BufReader<OwnedReadHalf>>,
    writer: Option<OwnedWriteHalf>,
    timeout: time::Duration,
}

impl Rig {
    /// Create a new instance of `Rig`.
    pub fn new(host: &str, port: u16) -> Rig {
        Rig {
            host: String::from(host),
            port,
            reader: None,
            writer: None,
            timeout: time::Duration::from_millis(250),
        }
    }

    /// Connect to a already running `rigctld`.
    pub async fn connect(&mut self) -> Result<(), RigError> {
        if self.is_connected() {
            return Err(RigError::AlreadyConnected);
        }

        let constring = format!("{}:{}", self.host, self.port);

        let stream = TcpStream::connect(constring)
            .await
            .map_err(|_| RigError::ConnectionError)?;
        let (rx, tx) = stream.into_split();
        self.reader = Some(BufReader::new(rx));
        self.writer = Some(tx);

        Ok(())
    }

    /// Disconnect from `rigctld`.
    /// Returns true after disconnect. May return false in case the connection was already closed.
    pub fn disconnect(&mut self) -> bool {
        if self.is_connected() {
            self.reader = None;
            self.writer = None;
            true
        } else {
            false
        }
    }

    /// Set communication timeout for communication with `rigctld`.
    pub fn set_communication_timeout(&mut self, timeout: time::Duration) {
        self.timeout = timeout;
    }

    /// Check if connected to rig
    pub fn is_connected(&self) -> bool {
        self.reader.is_some() && self.writer.is_some()
    }

    /// Get the rigs frequency.
    ///
    /// # Arguments:
    ///
    /// (None)
    ///
    /// # Result
    ///
    /// Returns the frequency or in case of an error the error cause.
    pub async fn get_frequency(&mut self) -> Result<u64, RigError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^get_freq:;Frequency: (\d+);RPRT 0$").unwrap();
        }

        let response = self.execute_command(r";\get_freq").await?;
        let freq = RE
            .captures(&response)
            .map_or(Err(RigError::InternalError), |c| Ok(c.get(1).unwrap()))?;
        let freq = freq.as_str().parse::<u64>().unwrap();

        Ok(freq)
    }

    /// Set the rigs frequency.
    ///
    /// # Arguments:
    ///
    /// * `frequency`: Frequency (Hz)
    ///
    /// # Result
    ///
    /// In case of an error the causing error is returned.
    pub async fn set_frequency(&mut self, frequency: u64) -> Result<(), RigError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^set_freq: (\d+);RPRT 0$").unwrap();
        }

        let request = format!(r";\set_freq {}", frequency);
        let response = self.execute_command(&request).await?;

        let freq = RE
            .captures(&response)
            .map_or(Err(RigError::InternalError), |c| Ok(c.get(1).unwrap()))?;
        let freq_out = freq.as_str().parse::<u64>().unwrap();

        if freq_out == frequency {
            Ok(())
        } else {
            Err(RigError::InternalError)
        }
    }

    /// Get the rigs mode.
    ///
    /// # Arguments:
    ///
    /// (None)
    ///
    /// # Result
    ///
    /// Returns the mode and passband or in case of an error the error cause.
    pub async fn get_mode(&mut self) -> Result<(Mode, u16), RigError> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^get_mode:;Mode: ([A-Z]+);Passband: (\d+);RPRT 0$").unwrap();
        }

        let response = self.execute_command(r";\get_mode").await?;
        let result = RE
            .captures(&response)
            .map_or(Err(RigError::InternalError), |c| {
                Ok((c.get(1).unwrap(), c.get(2).unwrap()))
            })?;
        let mode = Mode::from_str(result.0.as_str())?;
        let passband = result.1.as_str().parse::<u16>().unwrap();

        Ok((mode, passband))
    }

    /// Set the rigs mode.
    ///
    /// # Arguments:
    ///
    /// * `mode`: Operating mode
    /// * `passband`: Passband frequency (Hz)
    ///
    /// # Result
    ///
    /// In case of an error the causing error is returned.
    pub async fn set_mode(&mut self, mode: Mode, passband: u16) -> Result<(), RigError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^set_mode: ([A-Z]+) (\d+);RPRT 0$").unwrap();
        }

        let request = format!(r";\set_mode {} {}", mode, passband);
        let response = self.execute_command(&request).await?;

        let result = RE
            .captures(&response)
            .map_or(Err(RigError::InternalError), |c| {
                Ok((c.get(1).unwrap(), c.get(2).unwrap()))
            })?;
        let mode_out = Mode::from_str(result.0.as_str())?;
        let passband_out = result.1.as_str().parse::<u16>().unwrap();

        if mode == mode_out && passband_out == passband {
            Ok(())
        } else {
            Err(RigError::InternalError)
        }
    }

    /// Issue a command to rigctld and read its response.
    async fn execute_command(&mut self, input: &str) -> Result<String, RigError> {
        self.write_line(input).await?;
        self.read_line(self.timeout).await
    }

    /// Read a string from a tcp stream with timeout.
    async fn read_line(&mut self, timeout: time::Duration) -> Result<String, RigError> {
        let mut response = String::new();

        let res = time::timeout(
            timeout,
            self.reader.as_mut().unwrap().read_line(&mut response),
        )
        .await
        .map_err(|_| RigError::CommunicationTimeout)?;

        let _ = match res {
            Ok(0) => {
                self.reader = None;
                self.writer = None;
                Err(RigError::ConnectionLost)
            }
            Err(_) => Err(RigError::InternalError),
            Ok(num) => Ok(num),
        }?;

        response = String::from(response.trim_end());

        Ok(response)
    }

    /// Write a string to a tcp stream.
    /// Function appends '\n' to the given string before sending it.
    async fn write_line(&mut self, data: &str) -> Result<(), RigError> {
        self.writer
            .as_mut()
            .unwrap()
            .write_all(format!("{}\n", data).as_bytes())
            .await
            .map_err(|_| RigError::InternalError)
    }
}
