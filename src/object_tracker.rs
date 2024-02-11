use core::fmt::Debug;
use std::{
    cell::RefCell,
    ops::Deref,
    rc::{Rc, Weak},
};

use crate::gc::{Traverser, Visitor};

struct TrackedInner<T> {
    object: T,
    tracker: Weak<RefCell<ObjectTrackerInner<T>>>,
    id: usize,
}

impl<T> Drop for TrackedInner<T> {
    fn drop(&mut self) {
        if let Some(tracker) = self.tracker.upgrade() {
            if let Ok(mut tracker) = tracker.try_borrow_mut() {
                tracker.untrack(self.id);
            }
        }
    }
}

#[derive(Clone)]
pub struct Tracked<T>(Rc<TrackedInner<T>>);

impl<T> Deref for Tracked<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.object
    }
}

impl<T: Traverser> Traverser for Tracked<T> {
    fn traverse(&self, visitor: &Visitor) {
        visitor.traverse(&self.0.object, "Tracked");
    }
}

impl<T> Debug for Tracked<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Tracked").field(&self.0.object).finish()
    }
}

impl<T: PartialEq> PartialEq for Tracked<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.0.id == other.0.id {
            // It's the exact same object reference, so it must be equal.
            true
        } else {
            self.0.object == other.0.object
        }
    }
}

struct ObjectTrackerInner<T> {
    objects: Vec<Option<Weak<TrackedInner<T>>>>,
    /// Indexes into the `objects` vec that are None. Makes it easy to do
    /// constant-time creation of new objects, instead of having to traverse
    /// the vec to find one.
    free_objects: Vec<usize>,
}

impl<T> ObjectTrackerInner<T> {
    fn track(&mut self, object: T, weak_self: Weak<RefCell<Self>>) -> Tracked<T> {
        if let Some(id) = self.free_objects.pop() {
            let rc = Rc::new(TrackedInner {
                object,
                tracker: weak_self,
                id,
            });
            assert!(matches!(self.objects.get(id), Some(None)));
            self.objects[id] = Some(Rc::downgrade(&rc));
            Tracked(rc)
        } else {
            let id = self.objects.len();
            let rc = Rc::new(TrackedInner {
                object,
                tracker: weak_self,
                id,
            });
            self.objects.push(Some(Rc::downgrade(&rc)));
            Tracked(rc)
        }
    }

    fn untrack(&mut self, id: usize) {
        self.objects[id] = None;
        self.free_objects.push(id);
    }

    pub fn stats(&self) -> String {
        let allocated = self.objects.len();
        let free = self.free_objects.len();
        format!(
            "{} allocated, {} free, {} live",
            allocated,
            free,
            allocated - free
        )
    }
}

/// This struct makes it easy to keep track of how many
/// objects we have allocated.
///
/// Right now it's not terribly performant or space-efficent,
/// but at least it lets us know if we're leaking memory,
/// without requiring us to use a debugger.
pub struct ObjectTracker<T>(Rc<RefCell<ObjectTrackerInner<T>>>);

impl<T> Default for ObjectTracker<T> {
    fn default() -> Self {
        let inner = ObjectTrackerInner {
            objects: vec![],
            free_objects: vec![],
        };
        Self(Rc::new(RefCell::new(inner)))
    }
}

impl<T> ObjectTracker<T> {
    pub fn track(&mut self, object: T) -> Tracked<T> {
        let weak_self = Rc::downgrade(&self.0);
        self.0.borrow_mut().track(object, weak_self)
    }

    pub fn stats(&self) -> String {
        self.0.borrow().stats()
    }
}
