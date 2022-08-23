use tea_codec::{define_scope, errorx::single};
use wascc_codec::error::WasccCodec;

define_scope! {
    WasccHost: WasccCodec {
        wascap::Error => Wascap, "Embedded JWT Failure", @Debug;
        UnauthorizedCapability as v => UnauthorizedCapability, format!("Dispatch between actor and unauthorized capability: {} <-> {}", v.0, v.1), @Debug;
        CapabilityFailure as v => CapabilityFailure, v.0.to_string(), @Debug, single(&v.0);
        UnknownActor => UnknownActor, "Trying to call an unknown actor";
        HotSwapFailure => HotSwapFailure, "Failed to perform hot swap";
        ActorToActorCallNotExist => ActorToActorCallNotExist, "Attempted actor-to-actor call to non-existent target", @Debug;
        HostCallFailure as v=> HostCallFailure, v.0.to_string(), @Debug, single(&v.0);
        BadDispatch => BadDispatch, "Bad dispatch";
        NativeHostCannotBeActor => NativeHostCannotBeActor, "Invocation target of native host can't be actor";
        Authorization as v => Authorization, v.0.as_str();
        CapabilityProvider as v => CapabilityProvider, v.0.as_str();
        MiscHost as v => MiscHost, v.0.as_str();
        libloading::Error => Plugin, @Display, @Debug;
    }
}

#[derive(Debug)]
pub struct UnauthorizedCapability(pub String, pub String);

#[derive(Debug)]
pub struct CapabilityFailure(pub Error<WasccCodec>);

#[derive(Debug)]
pub struct UnknownActor;

#[derive(Debug)]
pub struct HotSwapFailure;

#[derive(Debug)]
pub struct ActorToActorCallNotExist;

#[derive(Debug)]
pub struct HostCallFailure(pub Error<WasccCodec>);

#[derive(Debug)]
pub struct BadDispatch;

#[derive(Debug)]
pub struct NativeHostCannotBeActor;

#[derive(Debug)]
pub struct Authorization(pub String);

#[derive(Debug)]
pub struct CapabilityProvider(pub String);

#[derive(Debug)]
pub struct MiscHost(pub String);
