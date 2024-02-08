use std::rc::Rc;

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

    pub fn try_as_rc_list(&self) -> Option<Rc<Vec<SourceValue>>> {
        let mut maybe_list = self.iter().cloned().collect::<Vec<SourceValue>>();
        if maybe_list.pop().unwrap().0 == Value::EmptyList {
            Some(maybe_list.into())
        } else {
            None
        }
    }
}

pub fn vec_to_list(mut values: Vec<SourceValue>) -> Value {
    if values.is_empty() {
        return Value::EmptyList;
    }
    let mut latest = Pair {
        car: Value::Undefined.into(),
        cdr: Value::EmptyList.into(),
    };
    values.reverse();
    let len = values.len();
    for (i, value) in values.into_iter().enumerate() {
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

        assert_eq!(
            list.iter().cloned().collect::<Vec<SourceValue>>(),
            vec![1.0.into(), 2.0.into(), Value::EmptyList.into(),]
        );
    }
}
