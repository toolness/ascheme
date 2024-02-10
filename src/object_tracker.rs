use core::fmt::Debug;
use std::{
    cell::RefCell,
    ops::Deref,
    rc::{Rc, Weak},
};

struct TrackedInner<T>(T, Weak<RefCell<ObjectTrackerInner<T>>>, usize);

impl<T> Drop for TrackedInner<T> {
    fn drop(&mut self) {
        // TODO: Tell the tracker to forget about us.
        if let Some(tracker) = self.1.upgrade() {}
    }
}

#[derive(Clone)]
pub struct Tracked<T>(Rc<TrackedInner<T>>);

impl<T> Deref for Tracked<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

impl<T> Debug for Tracked<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Tracked").field(&self.0 .0).finish()
    }
}

impl<T: PartialEq> PartialEq for Tracked<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 .0 == other.0 .0
    }
}

struct ObjectTrackerInner<T> {
    objects: Vec<Option<Weak<TrackedInner<T>>>>,
}

impl<T> ObjectTrackerInner<T> {
    fn track(&mut self, object: T, weak_self: Weak<RefCell<Self>>) -> Tracked<T> {
        let id = self.objects.len();
        let rc = Rc::new(TrackedInner(object, weak_self, id));
        self.objects.push(Some(Rc::downgrade(&rc)));
        Tracked(rc)
    }
}

pub struct ObjectTracker<T>(Rc<RefCell<ObjectTrackerInner<T>>>);

impl<T> Default for ObjectTracker<T> {
    fn default() -> Self {
        let inner = ObjectTrackerInner { objects: vec![] };
        Self(Rc::new(RefCell::new(inner)))
    }
}

impl<T> ObjectTracker<T> {
    pub fn track(&mut self, object: T) -> Tracked<T> {
        let weak_self = Rc::downgrade(&self.0);
        self.0.borrow_mut().track(object, weak_self)
    }

    pub fn compact(&mut self) {
        // TODO: We don't need this anymore, it can be removed I think.
    }

    pub fn len(&self) -> usize {
        self.0.borrow().objects.len()
    }
}
