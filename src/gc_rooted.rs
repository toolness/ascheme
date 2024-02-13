use std::ops::Deref;

use crate::{
    gc::{Traverser, Visitor},
    object_tracker::{CycleBreaker, ObjectTracker, Tracked},
};

/// This roots all tracked objects in the GC while they're in-scope
/// so they aren't collected.
pub struct GCRootManager<T: Traverser> {
    // This is a bit heavyweight for our needs, since it has a bunch of
    // GC-related functionality that we don't need, but it's easier than
    // implementing much of the same functionality from scratch.
    tracker: ObjectTracker<GCRooted<T>>,
}

// Not sure why, but derive(Default) doesn't seem to do the trick here.
impl<T: Traverser> Default for GCRootManager<T> {
    fn default() -> Self {
        Self {
            tracker: Default::default(),
        }
    }
}

impl<T: Traverser> GCRootManager<T> {
    pub fn root(&mut self, object: T) -> Tracked<GCRooted<T>> {
        self.tracker.track(GCRooted(object))
    }

    pub fn root_many(&mut self, objects: Vec<T>) -> Vec<Tracked<GCRooted<T>>> {
        objects
            .into_iter()
            .map(|object| self.root(object))
            .collect()
    }

    pub fn stats(&self) -> String {
        self.tracker.stats()
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
        // Nothing we can do about cycles--it's up to the objects we're wrapping
        // to break any that are found.
    }
}

impl<T: Traverser> Traverser for GCRootManager<T> {
    fn traverse(&self, visitor: &Visitor) {
        for tracked in self.tracker.all() {
            visitor.traverse(&tracked)
        }
    }
}
