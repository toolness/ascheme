use std::f64::INFINITY;

use crate::interpreter::{CallableContext, CallableResult};

use super::util::number_args;

pub fn get_builtins() -> super::Builtins {
    vec![
        ("<", less_than),
        ("<=", less_than_or_equal_to),
        (">", greater_than),
        (">=", greater_than_or_equal_to),
        ("=", numeric_eq),
    ]
}

fn less_than(mut ctx: CallableContext) -> CallableResult {
    let mut latest: f64 = -INFINITY;
    for number in number_args(&mut ctx)? {
        if number <= latest {
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

fn less_than_or_equal_to(mut ctx: CallableContext) -> CallableResult {
    let mut latest: f64 = -INFINITY;
    for number in number_args(&mut ctx)? {
        if number < latest {
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

fn greater_than(mut ctx: CallableContext) -> CallableResult {
    let mut latest: f64 = INFINITY;
    for number in number_args(&mut ctx)? {
        if number >= latest {
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

fn greater_than_or_equal_to(mut ctx: CallableContext) -> CallableResult {
    let mut latest: f64 = INFINITY;
    for number in number_args(&mut ctx)? {
        if number > latest {
            return Ok(false.into());
        }
        latest = number;
    }
    Ok(true.into())
}

fn numeric_eq(mut ctx: CallableContext) -> CallableResult {
    let numbers = number_args(&mut ctx)?;
    if numbers.len() < 2 {
        Ok(true.into())
    } else {
        let number = numbers[0];
        for other_number in &numbers[1..] {
            if *other_number != number {
                return Ok(false.into());
            }
        }
        Ok(true.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::test_eval_success;

    #[test]
    fn less_than_works() {
        test_eval_success("(<)", "#t");
        test_eval_success("(< 1)", "#t");
        test_eval_success("(< 1 0)", "#f");
        test_eval_success("(< 0 1)", "#t");
        test_eval_success("(< 1 1)", "#f");
        test_eval_success("(< 0 1 2)", "#t");
        test_eval_success("(< 0 1 2 3 1)", "#f");
    }

    #[test]
    fn less_than_or_equal_to_works() {
        test_eval_success("(<=)", "#t");
        test_eval_success("(<= 1)", "#t");
        test_eval_success("(<= 1 0)", "#f");
        test_eval_success("(<= 0 1)", "#t");
        test_eval_success("(<= 1 1)", "#t");
        test_eval_success("(<= 0 1 2)", "#t");
        test_eval_success("(<= 0 1 2 3 1)", "#f");
        test_eval_success("(<= 0 1 1 1 1)", "#t");
    }

    #[test]
    fn greater_than_works() {
        test_eval_success("(>)", "#t");
        test_eval_success("(> 1)", "#t");
        test_eval_success("(> 1 0)", "#t");
        test_eval_success("(> 0 1)", "#f");
        test_eval_success("(> 1 1)", "#f");
        test_eval_success("(> 2 1 0)", "#t");
        test_eval_success("(< 3 2 1 0 1)", "#f");
    }

    #[test]
    fn greater_than_or_equal_to_works() {
        test_eval_success("(>=)", "#t");
        test_eval_success("(>= 1)", "#t");
        test_eval_success("(>= 1 0)", "#t");
        test_eval_success("(>= 0 1)", "#f");
        test_eval_success("(>= 1 1)", "#t");
        test_eval_success("(>= 2 1 0)", "#t");
        test_eval_success("(<= 3 2 1 0 1)", "#f");
        test_eval_success("(>= 2 1 1 1 1)", "#t");
    }

    #[test]
    fn numeric_eq_works() {
        test_eval_success("(=)", "#t");
        test_eval_success("(= 1)", "#t");
        test_eval_success("(= 1 0)", "#f");
        test_eval_success("(= 1 1)", "#t");
        test_eval_success("(= 1 1 1)", "#t");
        test_eval_success("(= 1 2 3 4)", "#f");
    }
}
