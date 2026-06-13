use crate::context::Context;
use crate::param::{ParamSpec, ParamValue};

pub trait Node: NodeClone + Send {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn description(&self) -> &'static str {
        ""
    }
    fn params(&self) -> &'static [ParamSpec];
    fn get_param(&self, id: &str) -> Option<ParamValue>;
    fn set_param(&mut self, id: &str, value: ParamValue);
    fn process(&mut self, input: f32, ctx: &Context) -> f32;
    fn reset(&mut self) {}
    fn is_source(&self) -> bool {
        false
    }
}

pub trait NodeClone {
    fn clone_box(&self) -> Box<dyn Node>;
}

impl<T: Node + Clone + 'static> NodeClone for T {
    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Node> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
