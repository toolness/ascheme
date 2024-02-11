use core::fmt::Debug;
use std::{
    cell::RefCell,
    ops::Deref,
    rc::{Rc, Weak},
};

use crate::gc::{Traverser, Visitor};

struct TrackedInner<T: CycleBreaker> {
    object: T,
    tracker: Weak<RefCell<ObjectTrackerInner<T>>>,
    is_reachable: RefCell<bool>,
    has_had_cycles_broken: RefCell<bool>,
    id: usize,
}

impl<T: CycleBreaker> TrackedInner<T> {
    fn has_had_cycles_broken(&self) -> bool {
        *self.has_had_cycles_broken.borrow().deref()
    }

    fn is_reachable(&self) -> bool {
        *self.is_reachable.borrow().deref()
    }

    fn begin_mark(&self) {
        *self.is_reachable.borrow_mut() = false;
    }

    fn break_cycles(&self) {
        self.object.break_cycles();
        *self.has_had_cycles_broken.borrow_mut() = true;
    }
}

impl<T: CycleBreaker> Drop for TrackedInner<T> {
    fn drop(&mut self) {
        if let Some(tracker) = self.tracker.upgrade() {
            if let Ok(mut tracker) = tracker.try_borrow_mut() {
                tracker.untrack(self.id);
            } else if !std::thread::panicking() {
                eprintln!(
                    "WARNING: Unable to untrack object #{} (tracker.borrow_mut() failed).",
                    self.id
                );
            }
        } else if !std::thread::panicking() {
            eprintln!(
                "WARNING: Unable to untrack object #{} (tracker does not exist).",
                self.id
            );
        }
    }
}

#[derive(Clone)]
pub struct Tracked<T: CycleBreaker>(Rc<TrackedInner<T>>);

impl<T: CycleBreaker> Tracked<T> {
    pub fn mark_as_reachable(&self) {
        *self.0.is_reachable.borrow_mut() = true;
    }
}

impl<T: CycleBreaker> Deref for Tracked<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        if self.0.has_had_cycles_broken() {
            panic!(
                "Accessing object #{}, which has had its cycles broken.",
                self.0.id
            );
        }
        &self.0.object
    }
}

impl<T: Traverser + CycleBreaker> Traverser for Tracked<T> {
    fn traverse(&self, visitor: &Visitor) {
        self.mark_as_reachable();
        visitor.visit(&self.0.object, self.0.object.debug_name());
    }
}

impl<T: CycleBreaker> Debug for Tracked<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Tracked").field(&self.0.object).finish()
    }
}

impl<T: PartialEq + CycleBreaker> PartialEq for Tracked<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.0.id == other.0.id {
            // It's the exact same object reference, so it must be equal.
            true
        } else {
            self.0.object == other.0.object
        }
    }
}

struct ObjectTrackerInner<T: CycleBreaker> {
    objects: Vec<Option<Weak<TrackedInner<T>>>>,
    /// Indexes into the `objects` vec that are None. Makes it easy to do
    /// constant-time creation of new objects, instead of having to traverse
    /// the vec to find one.
    free_objects: Vec<usize>,
}

impl<T: CycleBreaker> ObjectTrackerInner<T> {
    fn track(&mut self, object: T, weak_self: Weak<RefCell<Self>>) -> Tracked<T> {
        if let Some(id) = self.free_objects.pop() {
            let rc = Rc::new(TrackedInner {
                object,
                tracker: weak_self,
                id,
                is_reachable: false.into(),
                has_had_cycles_broken: false.into(),
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
                is_reachable: false.into(),
                has_had_cycles_broken: false.into(),
            });
            self.objects.push(Some(Rc::downgrade(&rc)));
            Tracked(rc)
        }
    }

    fn untrack(&mut self, id: usize) {
        self.objects[id] = None;
        self.free_objects.push(id);
    }

    fn begin_mark(&mut self) {
        for obj in &self.objects {
            if let Some(obj) = obj {
                if let Some(obj) = obj.upgrade() {
                    obj.begin_mark();
                }
            }
        }
    }

    fn sweep(&mut self) -> Vec<Rc<TrackedInner<T>>> {
        let mut objs_in_cycles = vec![];
        for obj in &self.objects {
            if let Some(obj) = obj {
                if let Some(obj) = obj.upgrade() {
                    if !obj.is_reachable() {
                        objs_in_cycles.push(obj);
                    }
                }
            }
        }
        for obj in objs_in_cycles.iter() {
            obj.as_ref().break_cycles();
        }
        // Note that we're returning these in part because we don't want to
        // drop them: if we did, their `drop` methods would attempt to access us,
        // and we're already mutably borrowed!
        objs_in_cycles
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
pub struct ObjectTracker<T: CycleBreaker>(Rc<RefCell<ObjectTrackerInner<T>>>);

impl<T: CycleBreaker> Default for ObjectTracker<T> {
    fn default() -> Self {
        let inner = ObjectTrackerInner {
            objects: vec![],
            free_objects: vec![],
        };
        Self(Rc::new(RefCell::new(inner)))
    }
}

impl<T: CycleBreaker> ObjectTracker<T> {
    pub fn track(&mut self, object: T) -> Tracked<T> {
        let weak_self = Rc::downgrade(&self.0);
        self.0.borrow_mut().track(object, weak_self)
    }

    pub fn begin_mark(&mut self) {
        self.0.borrow_mut().begin_mark();
    }

    pub fn sweep(&mut self) -> usize {
        // Note that we need to capture the result in a variable, or
        // else the objects in broken cycles won't be able to
        // borrow the tracker to notify it that they've been dropped.
        let objs_in_cycles = self.0.borrow_mut().sweep();
        objs_in_cycles.len()
    }

    pub fn stats(&self) -> String {
        self.0.as_ref().borrow().stats()
    }
}

pub trait CycleBreaker {
    fn debug_name(&self) -> &'static str;

    fn break_cycles(&self);
}
