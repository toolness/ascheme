use std::ops::Deref;

use crate::{
    gc::{Traverser, Visitor},
    object_tracker::{CycleBreaker, ObjectTracker, Tracked},
};

pub struct GCRootManager<T: Traverser> {
    tracker: ObjectTracker<GCRooted<T>>,
}

impl<T: Traverser> Default for GCRootManager<T> {
    fn default() -> Self {
        Self {
            tracker: Default::default(),
        }
    }
}

impl<T: Traverser> GCRootManager<T> {
    pub fn root(&mut self, expressions: Vec<T>) -> Vec<Tracked<GCRooted<T>>> {
        expressions
            .into_iter()
            .map(|expr| self.tracker.track(GCRooted(expr)))
            .collect()
    }
}

pub struct GCRooted<T: Traverser>(T);

impl<T: Traverser> Traverser for GCRooted<T> {
    fn traverse(&self, visitor: &Visitor) {
        visitor.traverse(&self.0)
    }
}

impl<T: Traverser> Deref for GCRooted<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Traverser> CycleBreaker for GCRooted<T> {
    fn debug_name(&self) -> &'static str {
        return "GCRooted";
    }

    fn break_cycles(&self) {
        // Nothing we can do about cycles...
    }
}

impl<T: Traverser> Traverser for GCRootManager<T> {
    fn traverse(&self, visitor: &Visitor) {
        for tracked in self.tracker.all() {
            visitor.traverse(&tracked)
        }
    }
}
