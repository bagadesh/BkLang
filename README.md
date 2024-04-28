# The BLang Programming Language

This repository contains blang compiler.


## BLang Syntax
```rust

fn main() -> i32 {
    let a = 10; // Variable initialization
    let b = a; // Variable reassignment
    let c = 10 * 10 - 2 / 2 + 20; // Add, Subtract, Multiply, Division. BODMAS Rule
    let d = a * 2; // Expression parsinging with variables
    let y = foo();
    let z = bar();
    return z + y; // Return Statement
}

fn foo() -> i32 {
    let x = 20;
    if( x - 1 ) { // If consition with >=1 as true 0 as false
        //... contents
    } else if( x + 1 ) { // else if condition 
        //... contents 
    } else { // Else condition
        //... contents
    }
    return bar(); // Function call inside function
}

fn bar() -> i32 {
    return 100;
}

```

## Supported

- Supports only `Aarch64`. (Hand rolled it 😅)
- Supports only `int` type as default.
- Supports Function (without arguments)

## Planning

- Boolean
- If condition boolean check
- Function Arguments


If you haven't noticed it is really a toy language for me to experiment.



