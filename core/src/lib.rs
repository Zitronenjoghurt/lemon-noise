#[cfg(feature = "audio")]
mod audio;
mod context;
mod mixer;
mod modulation;
mod node;
pub mod nodes;
mod param;
#[cfg(feature = "persistence")]
pub mod persistence;
mod pipeline;
#[cfg(feature = "serde")]
mod project;
mod registry;
mod rng;
mod wav;

#[cfg(feature = "audio")]
pub use audio::{AudioEngine, AudioError};
pub use context::Context;
pub use mixer::{Channel, Mixer};
pub use modulation::{AutomationPoint, Binding, Interp, LfoShape, ModMode, Modulator};
pub use node::Node;
pub use param::{ParamKind, ParamSpec, ParamValue};
pub use pipeline::Pipeline;
#[cfg(feature = "serde")]
pub use project::{BindingState, ChannelState, NodeState, ProjectState};
pub use registry::{NodeDescriptor, Registry};
pub use rng::Rng;
pub use wav::{WavFormat, encode_wav};

pub const DEFAULT_SAMPLE_RATE: u32 = 44_100;
