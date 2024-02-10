use core::fmt::Debug;
use std::{
    ops::Deref,
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub struct Tracked<T>(Rc<T>);

impl<T> Deref for Tracked<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Debug for Tracked<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Tracked").field(&self.0).finish()
    }
}

impl<T: PartialEq> PartialEq for Tracked<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub struct ObjectTracker<T> {
    objects: Vec<Weak<T>>,
}

impl<T> Default for ObjectTracker<T> {
    fn default() -> Self {
        Self { objects: vec![] }
    }
}

impl<T> ObjectTracker<T> {
    pub fn track(&mut self, object: T) -> Tracked<T> {
        let rc = Rc::new(object);
        self.objects.push(Rc::downgrade(&rc));
        Tracked(rc)
    }

    pub fn compact(&mut self) {
        let objects = std::mem::take(&mut self.objects);
        self.objects = objects
            .into_iter()
            .filter(|weakref| weakref.upgrade().is_some())
            .collect();
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }
}
