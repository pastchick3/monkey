# Monkey Programming Language in Rust

It is an implementation of the Monkey Programming Language described in [Writing an Interpreter in Go](https://interpreterbook.com/) and [Writing a Compiler in Go](https://compilerbook.com/) by Thorsten Ball, which is also my first project using Rust.

- Interpreter

    - Builtin functions are not implemented, because currently I cannot find a way to store closures in enums.

    - The hash is not implemented, because I do not figure out how to implement Hash trait for expressions yet.

- Compiler & Virtual Machine

    - Except those mentioned in "Interpreter" part, closures are not implemented too. The direct reason is there were some stuff related to mut and borrow went wrong when I was trying to add free symbols into SymbolTable. What's worse, I soon realized that I did not treat global symbols specially, which means closures would capture global variables as well.

- Take-Home Lesson

    - Use short names. Generally there will be lots of wrapper types around the types you defined.

    - Think twice before hacking. Rust is a fairly rigorous language, you should have the big picture clearly in your head, and design ownership relations carefully before you start to implement.

    - `panic!` only absolutely necessary. Unlike languages using exceptions, in Rust, you cannot recover from panic. If there are something users can fix, use `Result` instead.

- Have Some Fun

    ```
        PS C:\Users\33160\Desktop> ./monkey
        Welcome to the Monkey Programming Language in Rust! (Interpreter)
        >> let arr = [1, "2", true];
        Null
        >> let adder = fn(a) { fn(b) { a + b;}; };
        Null
        >> let add_2 = adder(2);
        Null
        >> add_2(arr[0]);
        3

    ```

Or type `./monkey vm` to use the compiler & vitual machine!