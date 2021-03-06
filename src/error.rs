//
// Copyright 2018-2019 Tamas Blummer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
//!
//! # SPV Error
//!
//! All modules of this library use this error class to indicate problems.
//!

use hammersbald::HammersbaldError;

use rusqlite;
use bitcoin::util;
use std::convert;
use std::error::Error;
use std::fmt;
use std::io;
use bitcoin::consensus::encode;

/// An error class to offer a unified error interface upstream
pub enum SPVError {
    /// bad proof of work
    SpvBadProofOfWork,
    /// unconnected header chain detected
    UnconnectedHeader,
    /// no chain tip found
    NoTip,
    /// no peers to connect to
    NoPeers,
    /// unknown UTXO referred
    UnknownUTXO,
    /// Merkle root of block does not match the header
    BadMerkleRoot,
    /// downstream error
    Downstream(String),
    /// Network IO error
    IO(io::Error),
    /// Database error
    DB(rusqlite::Error),
    /// Bitcoin util error
    Util(util::Error),
    /// Bitcoin serialize error
    Serialize(encode::Error),
    /// Hammersbald error
    Hammersbald(HammersbaldError)
}

impl Error for SPVError {
    fn description(&self) -> &str {
        match *self {
            SPVError::SpvBadProofOfWork => "bad proof of work",
            SPVError::UnconnectedHeader => "unconnected header",
            SPVError::NoTip => "no chain tip found",
            SPVError::UnknownUTXO => "unknown utxo",
            SPVError::NoPeers => "no peers",
            SPVError::BadMerkleRoot => "merkle root of header does not match transaction list",
            SPVError::Downstream(ref s) => s,
            SPVError::IO(ref err) => err.description(),
            SPVError::DB(ref err) => err.description(),
            SPVError::Util(ref err) => err.description(),
            SPVError::Hammersbald(ref err) => err.description(),
            SPVError::Serialize(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            SPVError::SpvBadProofOfWork => None,
            SPVError::UnconnectedHeader => None,
            SPVError::NoTip => None,
            SPVError::NoPeers => None,
            SPVError::UnknownUTXO => None,
            SPVError::Downstream(_) => None,
            SPVError::BadMerkleRoot => None,
            SPVError::IO(ref err) => Some(err),
            SPVError::DB(ref err) => Some(err),
            SPVError::Util(ref err) => Some(err),
            SPVError::Hammersbald(ref err) => Some(err),
            SPVError::Serialize(ref err) => Some(err)
        }
    }
}

impl fmt::Display for SPVError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            SPVError::SpvBadProofOfWork |
            SPVError::UnconnectedHeader |
            SPVError::NoTip |
            SPVError::NoPeers | SPVError::BadMerkleRoot |
            SPVError::UnknownUTXO => write!(f, "{}", self.description()),
            SPVError::Downstream(ref s) => write!(f, "{}", s),
            SPVError::IO(ref err) => write!(f, "IO error: {}", err),
            SPVError::DB(ref err) => write!(f, "DB error: {}", err),
            SPVError::Util(ref err) => write!(f, "Util error: {}", err),
            SPVError::Hammersbald(ref err) => write!(f, "Hammersbald error: {}", err),
            SPVError::Serialize(ref err) => write!(f, "Serialize error: {}", err),
        }
    }
}

impl fmt::Debug for SPVError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self as &fmt::Display).fmt(f)
    }
}

impl convert::From<SPVError> for io::Error {
    fn from(err: SPVError) -> io::Error {
        match err {
            SPVError::IO(e) => e,
            _ => io::Error::new(io::ErrorKind::Other, err.description())
        }
    }
}

impl convert::From<io::Error> for SPVError {
    fn from(err: io::Error) -> SPVError {
        SPVError::IO(err)
    }
}


impl convert::From<util::Error> for SPVError {
    fn from(err: util::Error) -> SPVError {
        SPVError::Util(err)
    }
}

impl convert::From<rusqlite::Error> for SPVError {
    fn from(err: rusqlite::Error) -> SPVError {
        SPVError::DB(err)
    }
}

impl convert::From<HammersbaldError> for SPVError {
    fn from(err: HammersbaldError) -> SPVError {
        SPVError::Hammersbald(err)
    }
}

impl convert::From<encode::Error> for SPVError {
    fn from(err: encode::Error) -> SPVError {
        SPVError::Serialize(err)
    }
}

impl convert::From<Box<Error>> for SPVError {
    fn from(err: Box<Error>) -> Self {
        SPVError::Downstream(err.description().to_owned())
    }
}