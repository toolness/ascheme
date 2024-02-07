#[derive(Debug, PartialEq, Clone)]
pub enum Datum {
    Number(f64),
    EmptyList,
    Pair(Box<Pair>),
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
    current: Option<Box<Pair>>,
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

        let result = pair.car;
        if let Datum::Pair(pair) = pair.cdr {
            self.current = Some(pair);
        } else {
            self.last = Some(pair.cdr);
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
