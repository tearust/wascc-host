//! Custom error types

// Copyright 2015-2020 Capital One Services, LLC
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

use std::error::Error as StdError;
use std::fmt;
use tea_codec::error::code::common::{new_common_error_code, STD_IO_ERROR};
use tea_codec::error::code::wascc::{
    new_wascc_error_code, CAPABILITY_PROVIDER_ERROR, HOST_AUTHORIZATION_ERROR, HOST_CALL_FAILURE,
    MIDDLEWARE_ERROR, MISC_HOST_ERROR, PLUGIN_ERROR, WAPC_GENERAL_ERROR, WASCAP_GENERAL_ERROR,
};
use tea_codec::error::TeaError;
use wapc::errors::WapcError;

#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

pub fn new(kind: ErrorKind) -> Error {
    Error(Box::new(kind))
}

#[derive(Debug)]
pub enum ErrorKind {
    Wapc(WapcError),
    HostCallFailure(TeaError),
    Wascap(wascap::Error),
    Authorization(String),
    IO(std::io::Error),
    CapabilityProvider(String),
    MiscHost(String),
    Plugin(libloading::Error),
    Middleware(String),
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

impl Into<TeaError> for Error {
    fn into(self) -> TeaError {
        match *self.0 {
            ErrorKind::Wapc(inner) => {
                new_wascc_error_code(WAPC_GENERAL_ERROR).error_from_nested(inner.into())
            }
            ErrorKind::IO(e) => {
                new_common_error_code(STD_IO_ERROR).to_error_code(Some(format!("{:?}", e)), None)
            }
            ErrorKind::HostCallFailure(inner) => {
                new_wascc_error_code(HOST_CALL_FAILURE).error_from_nested(inner)
            }
            ErrorKind::Wascap(e) => {
                // todo transfer wascap error later
                new_wascc_error_code(WASCAP_GENERAL_ERROR)
                    .to_error_code(Some(format!("{:?}", e)), None)
            }
            ErrorKind::Authorization(s) => {
                new_wascc_error_code(HOST_AUTHORIZATION_ERROR).to_error_code(Some(s), None)
            }
            ErrorKind::CapabilityProvider(s) => {
                new_wascc_error_code(CAPABILITY_PROVIDER_ERROR).to_error_code(Some(s), None)
            }
            ErrorKind::MiscHost(s) => {
                new_wascc_error_code(MISC_HOST_ERROR).to_error_code(Some(s), None)
            }
            ErrorKind::Plugin(e) => {
                new_wascc_error_code(PLUGIN_ERROR).to_error_code(Some(format!("{:?}", e)), None)
            }
            ErrorKind::Middleware(s) => {
                new_wascc_error_code(MIDDLEWARE_ERROR).to_error_code(Some(s), None)
            }
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self.0 {
            ErrorKind::Wapc(_) => "waPC error",
            ErrorKind::IO(_) => "I/O error",
            ErrorKind::HostCallFailure(_) => "Error occurred during host call",
            ErrorKind::Wascap(_) => "Embedded JWT Failure",
            ErrorKind::Authorization(_) => "Module authorization failure",
            ErrorKind::CapabilityProvider(_) => "Capability provider failure",
            ErrorKind::MiscHost(_) => "waSCC Host error",
            ErrorKind::Plugin(_) => "Plugin error",
            ErrorKind::Middleware(_) => "Middleware error",
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self.0 {
            ErrorKind::Wapc(ref err) => Some(err),
            ErrorKind::HostCallFailure(_) => None,
            ErrorKind::Wascap(ref err) => Some(err),
            ErrorKind::Authorization(_) => None,
            ErrorKind::IO(ref err) => Some(err),
            ErrorKind::CapabilityProvider(_) => None,
            ErrorKind::MiscHost(_) => None,
            ErrorKind::Plugin(ref err) => Some(err),
            ErrorKind::Middleware(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorKind::Wapc(ref err) => write!(f, "waPC failure: {}", err),
            ErrorKind::HostCallFailure(ref err) => {
                write!(f, "Error occurred during host call: {:?}", err)
            }
            ErrorKind::Wascap(ref err) => write!(f, "Embedded JWT failure: {}", err),
            ErrorKind::Authorization(ref err) => {
                write!(f, "WebAssembly module authorization failure: {}", err)
            }
            ErrorKind::IO(ref err) => write!(f, "I/O error: {}", err),
            ErrorKind::CapabilityProvider(ref err) => {
                write!(f, "Capability provider error: {}", err)
            }
            ErrorKind::MiscHost(ref err) => write!(f, "waSCC Host Error: {}", err),
            ErrorKind::Plugin(ref err) => write!(f, "Plugin error: {}", err),
            ErrorKind::Middleware(ref err) => write!(f, "Middleware error: {}", err),
        }
    }
}

impl From<libloading::Error> for Error {
    fn from(source: libloading::Error) -> Error {
        Error(Box::new(ErrorKind::Plugin(source)))
    }
}
impl From<wascap::Error> for Error {
    fn from(source: wascap::Error) -> Error {
        Error(Box::new(ErrorKind::Wascap(source)))
    }
}

impl From<wapc::errors::WapcError> for Error {
    fn from(source: wapc::errors::WapcError) -> Error {
        Error(Box::new(ErrorKind::Wapc(source)))
    }
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Error {
        Error(Box::new(ErrorKind::IO(source)))
    }
}
