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
pub enum RigctldMode {
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

impl fmt::Display for RigctldMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RigctldMode::USB => write!(f, "USB"),
            RigctldMode::LSB => write!(f, "LSB"),
            RigctldMode::CW => write!(f, "CW"),
            RigctldMode::CWR => write!(f, "CWR"),
            RigctldMode::RTTY => write!(f, "RTTY"),
            RigctldMode::RTTYR => write!(f, "RTTYR"),
            RigctldMode::AM => write!(f, "AM"),
            RigctldMode::FM => write!(f, "FM"),
            RigctldMode::WFM => write!(f, "WFM"),
            RigctldMode::AMS => write!(f, "AMS"),
            RigctldMode::PKTLSB => write!(f, "PKTLSB"),
            RigctldMode::PKTUSB => write!(f, "PKTUSB"),
            RigctldMode::PKTFM => write!(f, "PKTFM"),
            RigctldMode::ECSSUSB => write!(f, "ECSSUSB"),
            RigctldMode::ECSSLSB => write!(f, "ECSSLSB"),
            RigctldMode::FAX => write!(f, "FAX"),
            RigctldMode::SAM => write!(f, "SAM"),
            RigctldMode::SAL => write!(f, "SAL"),
            RigctldMode::SAH => write!(f, "SAH"),
            RigctldMode::DSB => write!(f, "DSB"),
        }
    }
}

impl FromStr for RigctldMode {
    type Err = RigctldError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USB" => Ok(RigctldMode::USB),
            "LSB" => Ok(RigctldMode::LSB),
            "CW" => Ok(RigctldMode::CW),
            "CWR" => Ok(RigctldMode::CWR),
            "RTTY" => Ok(RigctldMode::RTTY),
            "RTTYR" => Ok(RigctldMode::RTTYR),
            "AM" => Ok(RigctldMode::AM),
            "FM" => Ok(RigctldMode::FM),
            "WFM" => Ok(RigctldMode::WFM),
            "AMS" => Ok(RigctldMode::AMS),
            "PKTLSB" => Ok(RigctldMode::PKTLSB),
            "PKTUSB" => Ok(RigctldMode::PKTUSB),
            "PKTFM" => Ok(RigctldMode::PKTFM),
            "ECSSUSB" => Ok(RigctldMode::ECSSUSB),
            "ECSSLSB" => Ok(RigctldMode::ECSSLSB),
            "FAX" => Ok(RigctldMode::FAX),
            "SAM" => Ok(RigctldMode::SAM),
            "SAL" => Ok(RigctldMode::SAL),
            "SAH" => Ok(RigctldMode::SAH),
            "DSB" => Ok(RigctldMode::DSB),
            _ => Err(RigctldError::InternalError),
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RigctldError {
    #[error("Connector error")]
    ConnectionError,

    #[error("Internal error")]
    InternalError,
}

/// Representation of a connection to rigctld.
pub struct Rigctld {
    /// Hostname
    pub host: String,

    /// Port
    pub port: u16,

    /// Tcp stream reading half
    reader: Option<BufReader<OwnedReadHalf>>,

    /// Tcp stream writing half
    writer: Option<OwnedWriteHalf>,
}

impl Rigctld {
    /// Create a new instance of `Rigctld`.
    pub fn new(host: &str, port: u16) -> Rigctld {
        Rigctld {
            host: String::from(host),
            port,
            reader: None,
            writer: None,
        }
    }

    /// Connect to a already running rigctld.
    pub async fn connect(&mut self) -> Result<(), RigctldError> {
        let constring = format!("{}:{}", self.host, self.port);

        let stream = TcpStream::connect(constring)
            .await
            .map_err(|_| RigctldError::ConnectionError)?;
        let (rx, tx) = stream.into_split();
        self.reader = Some(BufReader::new(rx));
        self.writer = Some(tx);

        Ok(())
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
    pub async fn get_frequency(&mut self) -> Result<u64, RigctldError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^get_freq:;Frequency: (\d+);RPRT 0$").unwrap();
        }

        let response = self.execute_command(r";\get_freq").await?;
        let freq = RE
            .captures(&response)
            .map_or(Err(RigctldError::InternalError), |c| Ok(c.get(1).unwrap()))?;
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
    pub async fn set_frequency(&mut self, frequency: u64) -> Result<(), RigctldError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^set_freq: (\d+);RPRT 0$").unwrap();
        }

        let request = format!(r";\set_freq {}", frequency);
        let response = self.execute_command(&request).await?;

        let freq = RE
            .captures(&response)
            .map_or(Err(RigctldError::InternalError), |c| Ok(c.get(1).unwrap()))?;
        let freq_out = freq.as_str().parse::<u64>().unwrap();

        if freq_out == frequency {
            Ok(())
        } else {
            Err(RigctldError::InternalError)
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
    pub async fn get_mode(&mut self) -> Result<(RigctldMode, u16), RigctldError> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^get_mode:;Mode: ([A-Z]+);Passband: (\d+);RPRT 0$").unwrap();
        }

        let response = self.execute_command(r";\get_mode").await?;
        let result = RE
            .captures(&response)
            .map_or(Err(RigctldError::InternalError), |c| {
                Ok((c.get(1).unwrap(), c.get(2).unwrap()))
            })?;
        let mode = RigctldMode::from_str(result.0.as_str())?;
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
    pub async fn set_mode(&mut self, mode: RigctldMode, passband: u16) -> Result<(), RigctldError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^set_mode: ([A-Z]+) (\d+);RPRT 0$").unwrap();
        }

        let request = format!(r";\set_mode {} {}", mode, passband);
        let response = self.execute_command(&request).await?;

        let result = RE
            .captures(&response)
            .map_or(Err(RigctldError::InternalError), |c| {
                Ok((c.get(1).unwrap(), c.get(2).unwrap()))
            })?;
        let mode_out = RigctldMode::from_str(result.0.as_str())?;
        let passband_out = result.1.as_str().parse::<u16>().unwrap();

        if mode == mode_out && passband_out == passband {
            Ok(())
        } else {
            Err(RigctldError::InternalError)
        }
    }

    /// Issue a command to rigctld and read its response.
    async fn execute_command(&mut self, input: &str) -> Result<String, RigctldError> {
        write_line(self.writer.as_mut().unwrap(), input).await?;
        read_line(self.reader.as_mut().unwrap()).await
    }
}

/// Read a string from a tcp stream.
async fn read_line(stream: &mut BufReader<OwnedReadHalf>) -> Result<String, RigctldError> {
    let mut response = String::new();

    let res = time::timeout(
        time::Duration::from_millis(1000),
        stream.read_line(&mut response),
    )
    .await
    .map_err(|_| RigctldError::ConnectionError)?;

    let _ = match res {
        Ok(0) => Err(RigctldError::ConnectionError),
        Err(_) => Err(RigctldError::InternalError),
        Ok(num) => Ok(num),
    }?;

    response = String::from(response.trim_end());

    Ok(response)
}

/// Write a string to a tcp stream.
/// Function appends '\n' to the given string befor sending it.
async fn write_line(stream: &mut OwnedWriteHalf, data: &str) -> Result<(), RigctldError> {
    stream
        .write_all(format!("{}\n", data).as_bytes())
        .await
        .map_err(|_| RigctldError::ConnectionError)
}
