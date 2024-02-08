use std::rc::Rc;

use crate::value::{SourceValue, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Pair {
    pub car: SourceValue,
    pub cdr: SourceValue,
}

impl Pair {
    pub fn into_iter(self) -> PairIterator {
        PairIterator {
            current: Some(self.into()),
            last: None,
        }
    }

    pub fn clone_and_try_into_rc_list(&self) -> Option<Rc<Vec<SourceValue>>> {
        self.clone().try_into_list().map(|list| list.into())
    }

    fn try_into_list(self) -> Option<Vec<SourceValue>> {
        let mut maybe_list = self.into_iter().collect::<Vec<SourceValue>>();
        if maybe_list.pop().unwrap().0 == Value::EmptyList {
            Some(maybe_list)
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

pub struct PairIterator {
    current: Option<Rc<Pair>>,
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

        let result = pair.car.clone();
        let cloned_cdr = pair.cdr.clone();
        if let Value::Pair(pair) = cloned_cdr.0 {
            self.current = Some(pair);
        } else {
            self.last = Some(cloned_cdr);
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
            car: Value::Number(1.0).into(),
            cdr: Value::Pair(
                Pair {
                    car: Value::Number(2.0).into(),
                    cdr: Value::EmptyList.into(),
                }
                .into(),
            )
            .into(),
        };

        assert_eq!(
            list.into_iter().collect::<Vec<SourceValue>>(),
            vec![
                Value::Number(1.0).into(),
                Value::Number(2.0).into(),
                Value::EmptyList.into(),
            ]
        );
    }
}
