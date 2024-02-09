use std::{collections::HashSet, rc::Rc};

use crate::value::{SourceValue, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Pair {
    pub car: SourceValue,
    pub cdr: SourceValue,
}

impl Pair {
    fn iter(&self) -> PairIterator {
        PairIterator {
            current: Some(&self),
            last: None,
        }
    }

    pub fn is_list(&self) -> bool {
        let mut latest = self;
        let mut visited: HashSet<*const Pair> = HashSet::new();
        loop {
            // TODO: Given current typings, I don't think it's actually
            // possible for cycles to exist in a Pair. But it *is* possible
            // in Scheme, and it might eventually be possible in this interpreter,
            // so we might as well add detection for them here.
            if visited.contains(&(latest as *const Pair)) {
                return false;
            }
            visited.insert(latest as *const Pair);
            match &latest.cdr.0 {
                Value::EmptyList => return true,
                Value::Pair(pair) => {
                    latest = pair.as_ref();
                }
                _ => return false,
            }
        }
    }

    pub fn try_as_rc_list(&self) -> Option<Rc<Vec<SourceValue>>> {
        if !self.is_list() {
            None
        } else {
            let mut list = self.iter().cloned().collect::<Vec<SourceValue>>();
            list.pop();
            Some(list.into())
        }
    }
}

pub fn vec_to_pair(mut initial_values: Vec<SourceValue>, final_value: SourceValue) -> Value {
    assert!(
        !initial_values.is_empty(),
        "vec_to_pair() must be given non-empty values!"
    );
    let mut latest = Pair {
        car: Value::Undefined.into(),
        cdr: final_value,
    };
    initial_values.reverse();
    let len = initial_values.len();
    for (i, value) in initial_values.into_iter().enumerate() {
        latest.car = value;
        if i < len - 1 {
            latest = Pair {
                car: Value::Undefined.into(),
                // TODO: Could probably come up with a better source map.
                cdr: Value::Pair(Rc::new(latest)).into(),
            }
        }
    }
    Value::Pair(Rc::new(latest))
}

pub fn vec_to_list(values: Vec<SourceValue>) -> Value {
    if values.is_empty() {
        return Value::EmptyList;
    }
    vec_to_pair(values, Value::EmptyList.into())
}

pub struct PairIterator<'a> {
    current: Option<&'a Pair>,
    last: Option<&'a SourceValue>,
}

impl<'a> Iterator for PairIterator<'a> {
    type Item = &'a SourceValue;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(pair) = self.current.take() else {
            return if let Some(last) = self.last.take() {
                Some(last)
            } else {
                None
            };
        };

        let result = &pair.car;
        if let Value::Pair(pair) = &pair.cdr.0 {
            self.current = Some(pair.as_ref());
        } else {
            self.last = Some(&pair.cdr);
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::value::Value;

    use super::{Pair, SourceValue};

    #[test]
    fn it_works() {
        let list = Pair {
            car: 1.0.into(),
            cdr: Value::Pair(
                Pair {
                    car: 2.0.into(),
                    cdr: Value::EmptyList.into(),
                }
                .into(),
            )
            .into(),
        };

        assert_eq!(list.is_list(), true);
        assert_eq!(
            list.iter().cloned().collect::<Vec<SourceValue>>(),
            vec![1.0.into(), 2.0.into(), Value::EmptyList.into(),]
        );
    }

    #[test]
    fn improper_lists_are_detected() {
        let improper_list = Pair {
            car: 1.0.into(),
            cdr: 2.0.into(),
        };
        assert_eq!(improper_list.is_list(), false);
    }
}
