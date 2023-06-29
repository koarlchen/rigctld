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
pub enum RigMode {
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

impl fmt::Display for RigMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RigMode::USB => write!(f, "USB"),
            RigMode::LSB => write!(f, "LSB"),
            RigMode::CW => write!(f, "CW"),
            RigMode::CWR => write!(f, "CWR"),
            RigMode::RTTY => write!(f, "RTTY"),
            RigMode::RTTYR => write!(f, "RTTYR"),
            RigMode::AM => write!(f, "AM"),
            RigMode::FM => write!(f, "FM"),
            RigMode::WFM => write!(f, "WFM"),
            RigMode::AMS => write!(f, "AMS"),
            RigMode::PKTLSB => write!(f, "PKTLSB"),
            RigMode::PKTUSB => write!(f, "PKTUSB"),
            RigMode::PKTFM => write!(f, "PKTFM"),
            RigMode::ECSSUSB => write!(f, "ECSSUSB"),
            RigMode::ECSSLSB => write!(f, "ECSSLSB"),
            RigMode::FAX => write!(f, "FAX"),
            RigMode::SAM => write!(f, "SAM"),
            RigMode::SAL => write!(f, "SAL"),
            RigMode::SAH => write!(f, "SAH"),
            RigMode::DSB => write!(f, "DSB"),
        }
    }
}

impl FromStr for RigMode {
    type Err = RigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USB" => Ok(RigMode::USB),
            "LSB" => Ok(RigMode::LSB),
            "CW" => Ok(RigMode::CW),
            "CWR" => Ok(RigMode::CWR),
            "RTTY" => Ok(RigMode::RTTY),
            "RTTYR" => Ok(RigMode::RTTYR),
            "AM" => Ok(RigMode::AM),
            "FM" => Ok(RigMode::FM),
            "WFM" => Ok(RigMode::WFM),
            "AMS" => Ok(RigMode::AMS),
            "PKTLSB" => Ok(RigMode::PKTLSB),
            "PKTUSB" => Ok(RigMode::PKTUSB),
            "PKTFM" => Ok(RigMode::PKTFM),
            "ECSSUSB" => Ok(RigMode::ECSSUSB),
            "ECSSLSB" => Ok(RigMode::ECSSLSB),
            "FAX" => Ok(RigMode::FAX),
            "SAM" => Ok(RigMode::SAM),
            "SAL" => Ok(RigMode::SAL),
            "SAH" => Ok(RigMode::SAH),
            "DSB" => Ok(RigMode::DSB),
            _ => Err(RigError::InternalError),
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RigError {
    /// Error in connection to `rigctld`
    #[error("Error in connection to `rigctld`")]
    ConnectionError,

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
        if self.reader.is_some() && self.writer.is_some() {
            return Err(RigError::ConnectionError);
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
    pub fn disconnect(&mut self) -> Result<(), RigError> {
        if self.reader.is_none() && self.writer.is_none() {
            Err(RigError::ConnectionError)
        } else {
            self.reader = None;
            self.writer = None;
            Ok(())
        }
    }

    /// Set communication timeout for communication with `rigctld`.
    pub fn set_communication_timeout(&mut self, timeout: time::Duration) {
        self.timeout = timeout;
    }

    /// Get frequency.
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

    /// Set frequency.
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

    /// Get mode.
    ///
    /// # Arguments:
    ///
    /// (None)
    ///
    /// # Result
    ///
    /// Returns the mode and passband or in case of an error the error cause.
    pub async fn get_mode(&mut self) -> Result<(RigMode, u16), RigError> {
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
        let mode = RigMode::from_str(result.0.as_str())?;
        let passband = result.1.as_str().parse::<u16>().unwrap();

        Ok((mode, passband))
    }

    /// Set mode.
    ///
    /// # Arguments:
    ///
    /// * `mode`: Operating mode
    /// * `passband`: Passband frequency (Hz)
    ///
    /// # Result
    ///
    /// In case of an error the causing error is returned.
    pub async fn set_mode(&mut self, mode: RigMode, passband: u16) -> Result<(), RigError> {
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
        let mode_out = RigMode::from_str(result.0.as_str())?;
        let passband_out = result.1.as_str().parse::<u16>().unwrap();

        if mode == mode_out && passband_out == passband {
            Ok(())
        } else {
            Err(RigError::InternalError)
        }
    }

    /// Issue a command to rigctld and read its response.
    async fn execute_command(&mut self, input: &str) -> Result<String, RigError> {
        write_line(self.writer.as_mut().unwrap(), input).await?;
        read_line(self.reader.as_mut().unwrap(), self.timeout).await
    }
}

/// Read a string from a tcp stream with timeout.
async fn read_line(
    stream: &mut BufReader<OwnedReadHalf>,
    timeout: time::Duration,
) -> Result<String, RigError> {
    let mut response = String::new();

    let res = time::timeout(timeout, stream.read_line(&mut response))
        .await
        .map_err(|_| RigError::ConnectionError)?;

    let _ = match res {
        Ok(0) => Err(RigError::ConnectionError),
        Err(_) => Err(RigError::InternalError),
        Ok(num) => Ok(num),
    }?;

    response = String::from(response.trim_end());

    Ok(response)
}

/// Write a string to a tcp stream.
/// Function appends '\n' to the given string befor sending it.
async fn write_line(stream: &mut OwnedWriteHalf, data: &str) -> Result<(), RigError> {
    stream
        .write_all(format!("{}\n", data).as_bytes())
        .await
        .map_err(|_| RigError::ConnectionError)
}
