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

#[derive(Debug, PartialEq, Eq)]
pub enum PowerState {
    PowerOff,
    PowerOn,
    StandBy,
}

impl FromStr for PowerState {
    type Err = RigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(PowerState::PowerOff),
            "1" => Ok(PowerState::PowerOn),
            "2" => Ok(PowerState::StandBy),
            _ => Err(RigError::InternalError),
        }
    }
}

impl fmt::Display for PowerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PowerState::PowerOff => write!(f, "0"),
            PowerState::PowerOn => write!(f, "1"),
            PowerState::StandBy => write!(f, "2"),
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

    /// Get powerstate.
    /// Important: current version of `rigctld` behaves not according to extended response protocol (see [here](https://github.com/Hamlib/Hamlib/pull/1324)).
    ///
    /// # Arguments:
    ///
    /// (None)
    ///
    /// # Result
    ///
    /// Returns the mode and passband or in case of an error the error cause.
    pub async fn get_powerstate(&mut self) -> Result<PowerState, RigError> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"^get_powerstat:;Power Status: (\d);RPRT 0$").unwrap();
        }

        let response = self.execute_command(r";\get_powerstat").await?;
        let result = RE
            .captures(&response)
            .map_or(Err(RigError::InternalError), |c| Ok(c.get(1).unwrap()))?;
        let passband = PowerState::from_str(result.as_str())?;

        Ok(passband)
    }

    /// Set power state.
    ///
    /// # Arguments:
    ///
    /// * `state`: Power state
    ///
    /// # Result
    ///
    /// In case of an error the causing error is returned.
    pub async fn set_powerstate(&mut self, state: &PowerState) -> Result<(), RigError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^set_powerstat: (\d);RPRT 0$").unwrap();
        }

        let request = format!(r";\set_powerstat {}", state);
        let response = self.execute_command(&request).await?;

        let result = RE
            .captures(&response)
            .map_or(Err(RigError::InternalError), |c| Ok(c.get(1).unwrap()))?;
        let state_out = PowerState::from_str(result.as_str()).unwrap();

        if *state == state_out {
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
