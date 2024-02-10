use std::cell::{Ref, RefCell};
use std::fmt::Display;
use std::ops::Deref;
use std::{collections::HashSet, rc::Rc};

use crate::value::{SourceValue, Value};

#[derive(Debug, PartialEq)]
pub enum VecPair {
    List(Rc<Vec<SourceValue>>),
    ImproperList(Rc<Vec<SourceValue>>),
}

impl Display for VecPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VecPair::List(items) => {
                write!(f, "(")?;
                let len = items.len();
                for (i, item) in items.iter().enumerate() {
                    write!(f, "{}", item)?;
                    if i < len - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")")
            }
            VecPair::ImproperList(items) => {
                write!(f, "(")?;
                let len = items.len();
                for (i, item) in items.iter().enumerate() {
                    write!(f, "{}", item)?;
                    if i == len - 2 {
                        write!(f, " . ")?;
                    } else if i < len - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PairType {
    List,
    ImproperList,
    Cyclic,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pair(Rc<RefCell<PairInner>>);

#[derive(Debug, PartialEq, Clone)]
pub struct PairInner {
    pub car: SourceValue,
    pub cdr: SourceValue,
}

impl From<PairInner> for Pair {
    fn from(value: PairInner) -> Self {
        Pair(Rc::new(RefCell::new(value)))
    }
}

impl Pair {
    fn inner(&self) -> Ref<PairInner> {
        self.0.borrow()
    }

    fn iter(&self) -> PairIterator {
        PairIterator {
            current: Some(self.clone()),
            last: None,
        }
    }

    fn as_list(&self) -> Vec<SourceValue> {
        let mut list = self.iter().collect::<Vec<SourceValue>>();
        list.pop();
        list.into()
    }

    pub fn try_get_vec_pair(&self) -> Option<VecPair> {
        match self.get_type() {
            PairType::List => Some(VecPair::List(self.as_list().into())),
            PairType::ImproperList => Some(VecPair::ImproperList(
                self.iter().collect::<Vec<SourceValue>>().into(),
            )),
            PairType::Cyclic => None,
        }
    }

    pub fn get_type(&self) -> PairType {
        let mut latest = self.clone();
        let mut visited: HashSet<*const PairInner> = HashSet::new();
        loop {
            if visited.contains(&(latest.inner().deref() as *const PairInner)) {
                return PairType::Cyclic;
            }
            visited.insert(latest.inner().deref() as *const PairInner);
            let new_latest = match &latest.inner().cdr.0 {
                Value::EmptyList => return PairType::List,
                Value::Pair(pair) => pair.clone(),
                _ => return PairType::ImproperList,
            };
            latest = new_latest;
        }
    }

    pub fn try_as_rc_list(&self) -> Option<Rc<Vec<SourceValue>>> {
        match self.get_type() {
            PairType::List => Some(self.as_list().into()),
            _ => None,
        }
    }
}

pub fn vec_to_pair(mut initial_values: Vec<SourceValue>, final_value: SourceValue) -> Value {
    assert!(
        !initial_values.is_empty(),
        "vec_to_pair() must be given non-empty values!"
    );
    let mut latest = PairInner {
        car: Value::Undefined.into(),
        cdr: final_value,
    };
    initial_values.reverse();
    let len = initial_values.len();
    for (i, value) in initial_values.into_iter().enumerate() {
        latest.car = value;
        if i < len - 1 {
            latest = PairInner {
                car: Value::Undefined.into(),
                // TODO: Could probably come up with a better source map.
                cdr: Value::Pair(latest.into()).into(),
            }
        }
    }
    Value::Pair(latest.into())
}

pub fn vec_to_list(values: Vec<SourceValue>) -> Value {
    if values.is_empty() {
        return Value::EmptyList;
    }
    vec_to_pair(values, Value::EmptyList.into())
}

pub struct PairIterator {
    current: Option<Pair>,
    last: Option<SourceValue>,
}

impl Iterator for PairIterator {
    type Item = SourceValue;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(pair) = self.current.take() else {
            return if let Some(last) = self.last.take() {
                Some(last)
            } else {
                None
            };
        };

        let result = pair.inner().car.clone();
        if let Value::Pair(pair) = &pair.inner().cdr.0 {
            self.current = Some(pair.clone());
        } else {
            self.last = Some(pair.inner().cdr.clone());
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{pair::PairType, value::Value};

    use super::{Pair, PairInner, SourceValue};

    fn pair(car: SourceValue, cdr: SourceValue) -> Pair {
        PairInner { car, cdr }.into()
    }

    #[test]
    fn it_works() {
        let list = pair(
            1.0.into(),
            Value::Pair(pair(2.0.into(), Value::EmptyList.into())).into(),
        );

        assert_eq!(list.get_type(), PairType::List);
        assert_eq!(
            list.iter().collect::<Vec<SourceValue>>(),
            vec![1.0.into(), 2.0.into(), Value::EmptyList.into(),]
        );
    }

    #[test]
    fn improper_lists_are_detected() {
        let improper_list = pair(1.0.into(), 2.0.into());
        assert_eq!(improper_list.get_type(), PairType::ImproperList);
    }
}
