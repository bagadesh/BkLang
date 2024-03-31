# The BLang Programming Language

This repository containers blang compiler.


## BLang Syntax
```rust

fn main() -> i32 {
    let a = 10;
    let b = 5;
    let x = 10;  // Variable initialization
    x = 20; // Variable reassignment
    x = 10 * 10 - 2 / 2 + 20; // Add, Subtract, Multiply, Division. BODMAS Rule
    x = x * 2; // Expression parsinging with variables
    return a + b; // Return Statement
}

fn foo() -> i32 {
    if( x - 1 ) { // If consition with >=1 as true 0 as false
        //... contents
    } else if( x + 1 ) { // else if condition 
        //... contents 
    } else { // Else condition
        //... contents
    }
    return a + b; // Return Statement
}

```

## Limitations

- Supports only `Aarch64`. (Hand rolled it 😅)
- No function support yet.
- Supports only `int` type as default.


If you haven't noticed it is really a toy language for me to experiment.



