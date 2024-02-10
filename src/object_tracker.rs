use std::rc::{Rc, Weak};

#[derive(Default)]
pub struct ObjectTracker<T> {
    objects: Vec<Weak<T>>,
}

impl<T> ObjectTracker<T> {
    pub fn track(&mut self, object: T) -> Rc<T> {
        let rc = Rc::new(object);
        self.objects.push(Rc::downgrade(&rc));
        rc
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
