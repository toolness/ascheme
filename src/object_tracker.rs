use std::{
    collections::HashMap,
    rc::{Rc, Weak},
};

pub struct Tracked<T>(Rc<T>, usize);

#[derive(Default)]
pub struct ObjectTracker<T> {
    objects: HashMap<usize, Weak<T>>,
    latest_id: usize,
}

impl<T> ObjectTracker<T> {
    pub fn track(&mut self, object: T) -> Tracked<T> {
        self.latest_id += 1;
        let id = self.latest_id;
        let rc = Rc::new(object);
        self.objects.insert(id, Rc::downgrade(&rc));
        Tracked(rc, id)
    }

    pub fn compact(&mut self) {
        let mut removed = Vec::with_capacity(self.objects.len());
        for (id, weakref) in self.objects.iter() {
            if weakref.upgrade().is_none() {
                removed.push(*id);
            }
        }
        for id in removed {
            self.objects.remove(&id);
        }
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }
}
