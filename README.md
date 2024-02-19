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

* Proper tail recursion
* Garbage collection (with caveats, see below)
* Lexical scoping
* Compound procedures
* Pairs
* Booleans
* Strings
* Floating-point numbers
* Symbols

## Limitations

There's a lot of things that haven't been implemented, some of which include:

* Exact/inexact numbers
* Rational, complex, and integer numbers (all numbers are just 64-bit floats)
* Radix prefixes for numbers
* Cyclic garbage collection can only be run manually from top-level code via the `gc` function.
  It can't be run when the call stack isn't empty.
* Symbol names, implemented as interned strings, aren't garbage collected.
* Characters
* Vectors
* Ports
* `call-with-current-continuation` (call/cc)

## Other resources

* [Revised<sup>5</sup> Report on the Algorithmic Language Scheme (R5RS) (PDF)](https://conservatory.scheme.org/schemers/Documents/Standards/R5RS/r5rs.pdf) - This is the specification I'm attempting to adhere to.

* [try.scheme.org](https://try.scheme.org/) - This is the main interpreter I use as a basis for determining how mine should behave, for anything not specified by R5RS.

## License

Everything in this repository not expressly attributed to other sources is licensed under [CC0 1.0 Universal](./LICENSE.md) (public domain).
