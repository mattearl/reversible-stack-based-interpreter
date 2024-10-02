use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
enum RuntimeError {
    DivideByZero,
    StackUnderflow,
    InvalidCommand,
    NoInstructions,
}

#[derive(Debug, Default)]
struct Interpreter {
    instructions: VecDeque<String>,
    stack: Vec<i32>,
    // Add any other state that you need here
}

impl Interpreter {
    /// Constructs a new interpreter with an empty list of instructions
    // and an empty stack.
    pub fn new() -> Self {
        todo!()
    }

    /// Adds instructions to the interpreter. The instructions are not
    // interpreted, just stored.
    ///
    /// interpreter.add_instructions(&[
    ///     "PUSH 1",
    ///     "PUSH 2",
    ///     "ADD",
    /// ]);
    pub fn add_instructions(&mut self, instructions: &[&str]) {
        todo!()
    }

    /// Returns a mutable reference to the next instruction that will be executed
    /// on the next `.forward()` call.
    pub fn current_instruction(&mut self) -> Option<&mut String> {
        todo!()
    }

    /// Interprets the first instruction in `Self.instructions`.
    /// If there are no instructions, returns `RuntimeError::NoInstructions`.
    /// Other errors should be handled as described in the `RuntimeError` struct.
    pub fn forward(&mut self) -> Result<(), RuntimeError> {
        todo!()
    }

    /// Calls `.forward()` until there are no more instructions or
    /// if there is an error.
    pub fn run(&mut self) -> Result<(), RuntimeError> {
        todo!()
    }

    /// *Reverses* the last instruction executed with `.forward()`.
    /// This should undo the last instruction and restore the state of
    /// the stack. Repeated calls should be possible until the stack
    /// is restored to its original state before the first forward call.
    ///
    /// If there is no instruction to reverse, return an error.
    pub fn back(&mut self) -> Result<(), RuntimeError> {
        todo!()
    }
}

fn main() {
    println!("Hello, world!");
}
