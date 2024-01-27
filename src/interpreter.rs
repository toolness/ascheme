use crate::{parser::Expression, string_interner::StringInterner};

pub struct Interpreter<'a> {
    expressions: &'a Vec<Expression>,
    interner: &'a mut StringInterner,
}

impl<'a> Interpreter<'a> {
    fn eval(&mut self) -> Option<f64> {
        for expression in self.expressions {
            // TODO
        }
        None
    }

    pub fn evaluate(
        expressions: &Vec<Expression>,
        interner: &'a mut StringInterner,
    ) -> Option<f64> {
        let mut interpreter = Interpreter {
            expressions,
            interner,
        };
        interpreter.eval()
    }
}
