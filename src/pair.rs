use std::rc::Rc;

use crate::value::{SourceValue, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Pair {
    pub car: SourceValue,
    pub cdr: SourceValue,
}

impl Pair {
    fn into_iter(self) -> PairIterator {
        PairIterator {
            current: Some(self.into()),
            last: None,
        }
    }
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
