# Evaluation Engine for Untyped Lambda Calculus

Rust package to parse, reduce, and print expressions of untyped lambda calculus, written for
educational purposes.

I wrote this to learn a bit of Rust without prior experience, so there is probably room for
improvement. It was mainly intended as an experiment to determine how variable binding and
substitution is best handled in Rust, in order to use that knowledge in another context.

## Running the Executable

Assuming Rust is installed, simply run
```
cargo run
```
or, to get a real performance boost:
```
cargo run -r
```

Each line read from standard input is parsed as a lambda expression, reduced (as much as possible,
but with a limit on the number of steps), and printed. The syntax is as usual, though `\` is
accepted as a substitute for `λ`. Multiple variables behind `λ` are supported, separated by
whitespace. All expressions must be closed.

For example, to multiply 2 and 3 as Church numerals, try:
```
(λ m n f x. m (n f) x)  (λ f x. f (f x))  (λ f x. f (f (f x)))
```

Deeply nested expressions currently cause stack overflows. (The cost of avoiding that seems too high
for such an educational project.)

## Implementation Details

As always, the main question when implementing lambda calculus is how to handle variables.

Abstractly speaking, one would really like every lambda expression to depend on some explicitly
specified context that defines which variables are available. In particular, when using De Bruijn
indices, the context defines the meaning of each index. Although an appropriate data structure
for a context can be constructed easily (see `context.rs`), it seems that the dependency of a lambda
expression on a context can really only be expressed in a dependently-typed language (and even then,
I don't actually know of any concrete implementation).

So while De Bruijn indices seem like a good fit for Rust (unlike, say, defining a variable reference
to be a pointer into an ancestor expression), manipulating them directly as numbers is a bit
error-prone. It seems like such operations should be part of the definition of a context, but in
reality, the data provided by a context is only needed when parsing or printing, but not when
reducing an expression. So right now, the reduction algorithm operates on indices directly. It would
be interesting to know if there is a more abstract solution.

In contrast to the purely functional way of reducing expressions, the code contains quite a lot
of mutation, mainly to avoid dynamic memory allocation as much as possible. In fact, the only
situation in which memory allocation happens during reduction is when an argument is applied to a
lambda abstraction whose variable occurs more than once. (Sharing is not used because in general
the De Bruijn indices in each copy must be shifted differently anyway.)

The resulting performance seems good, but I have not compared it with any other implementation, and
of course performance for particular expressions heavily depends on the evaluation strategy, which
is currently not configurable.

## Compile-Time Expressions

In addition to the run-time parser, a Rust macro `raw_expr!` implements the same syntax (within
constraints imposed by Rust tokenization) at compile time. This macro is used to produce the
built-in example expressions.
