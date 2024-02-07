use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum Datum {
    Number(f64),
    EmptyList,
    Pair(Rc<Pair>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pair {
    pub car: Datum,
    pub cdr: Datum,
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
    last: Option<Datum>,
}

impl Iterator for PairIterator {
    type Item = Datum;

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
        if let Datum::Pair(pair) = cloned_cdr {
            self.current = Some(pair);
        } else {
            self.last = Some(cloned_cdr);
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{Datum, Pair};

    #[test]
    fn it_works() {
        let list = Pair {
            car: Datum::Number(1.0),
            cdr: Datum::Pair(
                Pair {
                    car: Datum::Number(2.0),
                    cdr: Datum::EmptyList,
                }
                .into(),
            ),
        };

        assert_eq!(
            list.into_iter().collect::<Vec<Datum>>(),
            vec![Datum::Number(1.0), Datum::Number(2.0), Datum::EmptyList,]
        );
    }
}
