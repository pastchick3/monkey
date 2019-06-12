# Monkey Programming Language in Rust

- An implementation of the Monkey Programming Language described in [Writing an Interpreter in Go](https://interpreterbook.com/) by Thorsten Ball.
- Builtin functions are not implemented, because currently I cannot find a way to store closures in enums.
- The hash is not implemented, because I do not figure out how to implement Hash trait for expressions yet.
- Another flaw is that I used too much panic, so the interpreter will simply exit if I type anything wrong...