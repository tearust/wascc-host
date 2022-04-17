use crate::dispatch::WasccNativeDispatcher;
use crate::plugins::PluginManager;
use crate::router::Router;
use crate::{
    errors, middleware, router, Invocation, InvocationResponse, InvocationTarget, Middleware,
    NativeCapability, Result,
};
use crossbeam::channel;
use crossbeam_channel::{Receiver, Sender};
use crossbeam_utils::sync::WaitGroup;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use wascc_codec::capabilities::CapabilityDescriptor;

#[derive(Clone)]
pub struct NativeHost {
    plugins: Arc<RwLock<PluginManager>>,
    router: Arc<RwLock<Router>>,
    middlewares: Arc<RwLock<Vec<Box<dyn Middleware>>>>,
    caps: Arc<RwLock<HashMap<router::RouteKey, CapabilityDescriptor>>>,
}

impl NativeHost {
    pub fn new() -> Self {
        let host = NativeHost {
            router: Arc::new(RwLock::new(Router::default())),
            plugins: Arc::new(RwLock::new(PluginManager::default())),
            middlewares: Arc::new(RwLock::new(vec![])),
            caps: Arc::new(RwLock::new(HashMap::new())),
        };
        host.ensure_extras().unwrap();
        host
    }

    pub fn add_native_capability(&self, capability: NativeCapability) -> Result<()> {
        let capid = capability.id();
        if self
            .router
            .read()
            .unwrap()
            .route_exists(&capability.binding_name, &capability.id())
        {
            return Err(errors::new(errors::ErrorKind::CapabilityProvider(format!(
                "Capability provider {} cannot be bound to the same name ({}) twice, loading failed.", capid, capability.binding_name
            ))).into());
        }
        self.caps.write().unwrap().insert(
            (
                capability.binding_name.to_string(),
                capability.descriptor.id.to_string(),
            ),
            capability.descriptor().clone(),
        );
        let wg = crossbeam_utils::sync::WaitGroup::new();
        self.spawn_capability_provider_and_listen(capability, wg.clone())?;
        wg.wait();
        Ok(())
    }

    fn spawn_capability_provider_and_listen(
        &self,
        capability: NativeCapability,
        wg: WaitGroup,
    ) -> Result<()> {
        let capid = capability.id().to_string();
        let binding = capability.binding_name.to_string();
        let router = self.router.clone();
        let caps = self.caps.clone();

        self.plugins.write().unwrap().add_plugin(capability)?;
        let plugins = self.plugins.clone();
        let middlewares = self.middlewares.clone();

        thread::spawn(move || {
            let (inv_s, inv_r): (Sender<Invocation>, Receiver<Invocation>) = channel::unbounded();
            let (resp_s, resp_r): (Sender<InvocationResponse>, Receiver<InvocationResponse>) =
                channel::unbounded();
            let (term_s, term_r): (Sender<bool>, Receiver<bool>) = channel::unbounded();
            let dispatcher = WasccNativeDispatcher::new(resp_r.clone(), inv_s.clone(), &capid);
            plugins
                .write()
                .unwrap()
                .register_dispatcher(&binding, &capid, dispatcher)
                .unwrap();

            router
                .write()
                .unwrap()
                .add_route(&binding, &capid, inv_s, resp_r, term_s);

            info!("Native capability provider '({},{})' ready", binding, capid);
            drop(wg);

            loop {
                select! {
                    recv(inv_r) -> inv => {
                        if let Ok(ref inv) = inv {
                            let inv_r = match &inv.target {
                                InvocationTarget::Capability{capid: _tgt_capid, binding: _tgt_binding} => {
                                    // Run invocation through middleware, which will terminate at a plugin invocation
                                    middleware::invoke_capability(middlewares.clone(), plugins.clone(), router.clone(), inv.clone()).unwrap()
                                },
                                InvocationTarget::Actor(_) => {
                                   error!("## invocation target is actor");
                                   InvocationResponse::error(inv, "invocation target of native host can't be actor")
                                }
                            };
                            resp_s.send(inv_r).unwrap();
                        }
                    },
                    recv(term_r) -> _term => {
                        info!("Terminating native capability provider {},{}", binding, capid);
                        remove_cap(caps, &capid, &binding);
                        router.write().unwrap().remove_route(&binding, &capid);
                        plugins.write().unwrap().remove_plugin(&binding, &capid).unwrap();
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    fn ensure_extras(&self) -> Result<()> {
        if self
            .router
            .read()
            .unwrap()
            .get_route("default", "wascc:extras")
            .is_some()
        {
            return Ok(());
        }
        self.add_native_capability(NativeCapability::from_instance(
            crate::extras::ExtrasCapabilityProvider::default(),
            None,
        )?)?;
        Ok(())
    }
}

pub(crate) fn remove_cap(
    caps: Arc<RwLock<HashMap<crate::router::RouteKey, CapabilityDescriptor>>>,
    capid: &str,
    binding: &str,
) {
    caps.write()
        .unwrap()
        .remove(&(binding.to_string(), capid.to_string()));
}
