ASCHEME is a simple Rust-based Scheme interpreter.

This interpreter was made primarily for learning and personal edification, and
to work through [The Structure and Interpretation of Computer Programs (SICP)][SICP].

It is [missing features](#limitations) from mainstream implementations
of the language, so if you want a full-featured interpreter, you should probably
look elsewhere.

[SICP]: https://mitp-content-server.mit.edu/books/content/sectbyfn/books_pres_0/6515/sicp.zip/index.html

## Quick start

Install the executable with:

```
cargo install --path=.
```

You can run the Scheme interpreter interactively with:

```
ascheme
```

Or you can run a program:

```
ascheme programs/math.sch
```

Use `ascheme --help` for more details.

## What's implemented

Note that this list isn't exhaustive.

- Proper tail recursion
- Garbage collection (with caveats, see below)
- Lexical scoping
- `lambda`
- Compound procedures
- Pairs
- Booleans
- Strings
- Floating-point numbers
- Symbols

## Limitations

There's a lot of things that haven't been implemented, some of which include:

- Exact/inexact numbers
- Rational, complex, and integer numbers (all numbers are just 64-bit floats)
- Radix prefixes for numbers
- Cyclic garbage collection can only be run manually from top-level code via
  the `gc` function. It can't be run when the call stack isn't empty.
- Symbol names, implemented as interned strings, aren't garbage collected.
- Characters
- Vectors
- Ports
- Macros
- `call-with-current-continuation` (call/cc)
- `let`, `let*`, `letrec`

## Other notes, learnings, etc.

- Generally I'm implementing this interpreter as I work through SICP and read
  R5RS, so it's a work-in-progress. As an example, the earlier versions of the
  interpreter didn't have a concept of a mutable "pair" data type--only of a
  list, which was stored as an array rather than a linked list.

- Evaluation is performed via recursive descent, which leverages Rust's call
  stack to store program state, but this also means that the interpreter is
  bound by those constraints: it's unlikely I will ever be able to implement
  features like call/cc with this architecture.

### Homoiconicity

Early versions of this interpreter used completely separate data structures for
storing data and parsed code, but as I learned more about the language, I
unified them into one.

I think this means that my implementation is "homoiconic", in that the
interpreter's abstract syntax tree is literally the same thing as its
data. For reference, this unification was done in [`ccd0c69`](https://github.com/toolness/ascheme/commit/ccd0c69421bd082114eed55eb266c189b9457fc1).

### Proper tail recursion

- My implementation of proper tail recursion feels very similar to lazy
  evaluation: essentially, if a procedure is called in a tail context
  (as described by R5RS), instead of evaluating the expression and returning
  the result, it simply "bundles up" everything needed for the evaluation
  into a special struct and returns it (thereby popping rust's call stack).
  This struct is called a `BoundProcedure`.

  When evaluating expressions, the interpreter actually _loops_ until the
  result of the evaluation isn't a `BoundProcedure`. This is how recursion
  can become iteration.

  For reference, this was mostly implemented in [`d6f06e4`](https://github.com/toolness/ascheme/commit/d6f06e4aab168a54c9a33aefce32cd5881eb48da).

- A weird side effect of proper tail recursion is that, because all traces of
  the caller are removed from the call stack, it makes tracebacks incomplete.

  For example, take the following program:

  ```scheme
  (define (x) (y))
  (define (y) (z))
  (define (z) (kaboom))
  (z)
  ```

  As of 2024-07-21, the interpreter will produce this output when evaluating
  it:

  ```
  Error: UnboundVariable("kaboom" (#37)) in "programs/tail_recursion_error.sch", line 3:
  | (define (z) (kaboom))
  |              ^^^^^^
  Traceback (excluding tail calls, most recent call last):
    "programs/tail_recursion_error.sch", line 4:
    | (z)
    |  ^
  ```

  This makes determining causality difficult, and I'm curious if it's one
  of the reasons that most mainstream languages don't actually support proper
  tail recursion.

## Other resources

- [Revised<sup>5</sup> Report on the Algorithmic Language Scheme (R5RS) (PDF)](https://conservatory.scheme.org/schemers/Documents/Standards/R5RS/r5rs.pdf) - This is the specification I'm attempting to adhere to.

- [try.scheme.org](https://try.scheme.org/) - This is the main interpreter I use as a basis for determining how mine should behave, for anything not specified by R5RS.

## License

Everything in this repository not expressly attributed to other sources is licensed under [CC0 1.0 Universal](./LICENSE.md) (public domain).
