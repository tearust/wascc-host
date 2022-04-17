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

use crate::inthost::{Invocation, InvocationResponse, InvocationTarget};
use crossbeam_channel::{Receiver, Sender};
use tea_codec::error::code::common::{
    new_common_error_code, CHANNEL_RECEIVE_ERROR, CHANNEL_SEND_ERROR,
};
use tea_codec::error::code::wascc::{new_wascc_error_code, INVOCATION_ERROR};
use tea_codec::error::TeaResult;
use wascc_codec::capabilities::Dispatcher;

/// A dispatcher is given to each capability provider, allowing it to send
/// commands in to the guest module and await replies. This dispatch
/// is one way, and is _not_ used for the guest module to send commands to capabilities
#[derive(Clone)]
pub(crate) struct WasccNativeDispatcher {
    resp_r: Receiver<InvocationResponse>,
    invoc_s: Sender<Invocation>,
    capid: String,
}

impl WasccNativeDispatcher {
    pub fn new(
        resp_r: Receiver<InvocationResponse>,
        invoc_s: Sender<Invocation>,
        capid: &str,
    ) -> Self {
        WasccNativeDispatcher {
            resp_r,
            invoc_s,
            capid: capid.to_string(),
        }
    }
}

impl Dispatcher for WasccNativeDispatcher {
    /// Called by a capability provider to invoke a function on an actor
    fn dispatch(&self, actor: &str, op: &str, msg: &[u8]) -> TeaResult<Vec<u8>> {
        trace!(
            "Dispatching operation '{}' ({} bytes) to actor",
            op,
            msg.len()
        );
        let inv = Invocation::new(
            self.capid.to_string(),
            InvocationTarget::Actor(actor.to_string()),
            op,
            msg.to_vec(),
        );
        self.invoc_s.send(inv).map_err(|e| {
            new_common_error_code(CHANNEL_SEND_ERROR).to_error_code(Some(format!("{:?}", e)), None)
        })?;
        let resp = self.resp_r.recv();
        match resp {
            Ok(r) => match r.error {
                Some(s) => Err(new_wascc_error_code(INVOCATION_ERROR).to_error_code(Some(s), None)),
                None => Ok(r.msg),
            },
            Err(e) => Err(new_common_error_code(CHANNEL_RECEIVE_ERROR)
                .to_error_code(Some(format!("{:?}", e)), None)),
        }
    }
}
