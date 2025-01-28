1. What are Rust macros, and how are they different from functions? Provide an example of a simple macro.

Rust macros are functions for generating new rust code at compile time. 

```rs
// from std
use alloc::vec;
let empty_vec = vec![];

macro_rules! hello_world (
    () => {
        println!("hello world")
    }
);

hello_world!();
```

2. Explain the concept of "fearless concurrency" in Rust. How does the type system help achieve this?



3. What is the difference between #[derive] and manually implementing a trait like Debug or Clone?

`#[derive]` is a macro that can generate the code automatically if the underlying types in the struct/enum already implement `Clone`. either by bound or naturally.

manually implementing is manually writing the code for the trait impl. most of the times, you don't need that.

Part 5: Practical Coding
1. Write a Rust function that takes a vector of integers and returns the sum of all even numbers.

```rs
fn even_sum(ints: Vec<u128>) -> u128 {
    ints.iter().filter(|a| a % 2 == 0).sum()
}
```


2. Implement a simple struct called Person with fields name (String) and age (u8). Add a method to print the person’s details.

```rs

struct Person {
    name: String,
    age: u8,
}

impl Display for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Person name: {}, age: {}", self.name, self.age)
    }
}
```

3. Write a Rust program that reads a file and counts the number of words in it.

```rs
const FILE_NAME: &str = "name.txt";

let buffer = std::fs::read(FILE_NAME)?.to_string();

for 
```
