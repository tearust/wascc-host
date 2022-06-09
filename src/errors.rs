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

use tea_codec::error::{
    new_common_error_code, new_wascc_error_code, CommonCode, TeaError, WasccCode,
};
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
                new_wascc_error_code(WasccCode::WapcGeneralError).error_from_nested(inner.into())
            }
            ErrorKind::IO(e) => new_common_error_code(CommonCode::StdIoError)
                .to_error_code(Some(format!("{:?}", e)), None),
            ErrorKind::HostCallFailure(inner) => {
                new_wascc_error_code(WasccCode::HostCallFailure).error_from_nested(inner)
            }
            ErrorKind::Wascap(e) => {
                // todo transfer wascap error later
                new_wascc_error_code(WasccCode::WascapGeneralError)
                    .to_error_code(Some(format!("{:?}", e)), None)
            }
            ErrorKind::Authorization(s) => {
                new_wascc_error_code(WasccCode::HostAuthorizationError).to_error_code(Some(s), None)
            }
            ErrorKind::CapabilityProvider(s) => {
                new_wascc_error_code(WasccCode::CapabilityProviderError)
                    .to_error_code(Some(s), None)
            }
            ErrorKind::MiscHost(s) => {
                new_wascc_error_code(WasccCode::MiscHostError).to_error_code(Some(s), None)
            }
            ErrorKind::Plugin(e) => new_wascc_error_code(WasccCode::PluginError)
                .to_error_code(Some(format!("{:?}", e)), None),
            ErrorKind::Middleware(s) => {
                new_wascc_error_code(WasccCode::MiddlewareError).to_error_code(Some(s), None)
            }
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
