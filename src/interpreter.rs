//! This module implements a basic interpreter for stack-based instructions.
//! It provides the capability to execute arithmetic and stack manipulation instructions
//! with forward execution and backward undo functionality. The interpreter maintains
//! a history of executed instructions, allowing for state reversal. This module is
//! designed to support basic instructions like push, pop, and arithmetic operations
//! (addition, subtraction, multiplication, and division).
//!
//! # Examples
//!
//! ## Basic Arithmetic Operations
//!
//! ```rust
//! use reversible_interpreter::interpreter::Instruction;
//! use reversible_interpreter::interpreter::Interpreter;
//! let mut interpreter = Interpreter::new();
//! interpreter.add_instructions(&[
//!     Instruction::Push(5),
//!     Instruction::Push(3),
//!     Instruction::Add,
//! ]);
//! interpreter.run().unwrap();
//! assert_eq!(interpreter.stack(), &vec![8]);
//! ```
//! This example pushes 5 and 3 onto the stack, then adds them. The stack contains the result `8`.
//!
//! ## Undo (Backtrack) Operations
//!
//! ```rust
//! use reversible_interpreter::interpreter::Instruction;
//! use reversible_interpreter::interpreter::Interpreter;
//! let mut interpreter = Interpreter::new();
//! interpreter.add_instructions(&[
//!     Instruction::Push(4),
//!     Instruction::Push(2),
//!     Instruction::Mul,
//! ]);
//! interpreter.run().unwrap();
//! assert_eq!(interpreter.stack(), &vec![8]);
//! interpreter.back().unwrap();
//! assert_eq!(interpreter.stack(), &vec![4, 2]);
//! ```
//! Here, the multiplication of 4 and 2 is undone, restoring the stack to its previous state `[4, 2]`.
//!
//! ## Division and Error Handling
//!
//! ```rust
//! use reversible_interpreter::interpreter::Instruction;
//! use reversible_interpreter::interpreter::Interpreter;
//! let mut interpreter = Interpreter::new();
//! interpreter.add_instructions(&[
//!     Instruction::Push(10),
//!     Instruction::Push(0),
//!     Instruction::Div,
//! ]);
//! assert!(interpreter.run().is_err()); // Division by zero error.
//! assert_eq!(interpreter.stack(), &vec![10, 0]); // Stack is restored to previous state.
//! ```
//! In this example, the division of 10 by 0 results in a `DivideByZero` error, and the stack is not changed.

use std::collections::VecDeque;

/// Represents the possible instructions that can be executed by the interpreter.
///
/// - `Push(i32)`: Pushes an integer value onto the stack.
/// - `Pop`: Pops the top value off the stack.
/// - `Add`: Pops the top two values, adds them, and pushes the result.
/// - `Sub`: Pops the top two values, subtracts the second from the first, and pushes the result.
/// - `Mul`: Pops the top two values, multiplies them, and pushes the result.
/// - `Div`: Pops the top two values, divides the first by the second, and pushes the result.
///          If division by zero is attempted, it results in an error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Push(i32),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
}

/// Represents an entry in the execution history of the interpreter. Each entry records:
/// - The `instruction` that was executed.
/// - The values that were `popped_values` off the stack during the execution of the instruction.
/// - The values that were `pushed_values` onto the stack as a result of executing the instruction.
///
/// This structure is used to enable undo functionality in the interpreter by reversing
/// the stack changes for each executed instruction.
#[derive(Debug)]
struct HistoryEntry {
    instruction: Instruction,
    popped_values: Vec<i32>,
    pushed_values: Vec<i32>,
}

/// Represents possible runtime errors that can occur during the interpretation process.
///
/// - `DivideByZero`: Attempted to divide by zero.
/// - `StackUnderflow`: Tried to pop from an empty stack or use insufficient stack values.
/// - `NoInstructions`: No instructions available for execution.
/// - `ArithmeticOverflow`: An arithmetic operation caused an overflow.
/// - `InvalidCommand`: Encountered an unrecognized or malformed command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    DivideByZero,
    StackUnderflow,
    NoInstructions,
    ArithmeticOverflow,
    InvalidCommand,
}

/// The `Interpreter` struct manages the state of the stack-based instruction execution.
/// It holds:
/// - `instructions`: A queue of instructions to be executed.
/// - `stack`: A vector representing the current state of the stack.
/// - `history`: A list of past executions to allow for reversing instructions.
///
/// The interpreter supports forward execution of instructions and the ability to undo
/// previous operations via a backtracking mechanism.
#[derive(Debug, Default)]
pub struct Interpreter {
    instructions: VecDeque<Instruction>,
    stack: Vec<i32>,
    history: Vec<HistoryEntry>,
}

impl Interpreter {
    /// Constructs a new interpreter with an empty list of instructions
    /// and an empty stack.
    pub fn new() -> Self {
        Self {
            instructions: VecDeque::new(),
            stack: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Adds instructions to the interpreter. The instructions are not
    // interpreted, just stored.
    pub fn add_instructions(&mut self, instructions: &[Instruction]) {
        for instr in instructions {
            self.instructions.push_back(instr.clone());
        }
    }

    /// Returns a mutable reference to the next instruction that will be executed
    /// on the next `.forward()` call.
    pub fn current_instruction(&mut self) -> Option<&mut Instruction> {
        self.instructions.get_mut(0)
    }

    /// Returns a reference to the instruction queue.
    pub fn instructions(&self) -> &VecDeque<Instruction> {
        &self.instructions
    }

    /// Returns a reference to the stack.
    pub fn stack(&self) -> &Vec<i32> {
        &self.stack
    }

    /// Interprets the first instruction in `Self.instructions`.
    /// If there are no instructions, returns `RuntimeError::NoInstructions`.
    /// Other errors should be handled as described in the `RuntimeError` struct.
    pub fn forward(&mut self) -> Result<Instruction, RuntimeError> {
        // Remove the instruction from the queue
        let instruction = self
            .instructions
            .pop_front()
            .ok_or(RuntimeError::NoInstructions)?;

        match instruction {
            Instruction::Push(value) => {
                self.stack.push(value);
                self.history.push(HistoryEntry {
                    instruction: instruction.clone(),
                    popped_values: Vec::new(),
                    pushed_values: vec![value],
                });
                Ok(instruction)
            }
            Instruction::Pop => {
                let value = self.stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                self.history.push(HistoryEntry {
                    instruction: instruction.clone(),
                    popped_values: vec![value],
                    pushed_values: Vec::new(),
                });
                Ok(instruction)
            }
            Instruction::Add | Instruction::Sub | Instruction::Mul | Instruction::Div => {
                if self.stack.len() < 2 {
                    return Err(RuntimeError::StackUnderflow);
                }
                // The following pops should never fail since we already checked for underflow above.
                let b = self.stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                let a = self.stack.pop().ok_or(RuntimeError::StackUnderflow)?;

                let result = match instruction {
                    Instruction::Add => a.checked_add(b),
                    Instruction::Sub => a.checked_sub(b),
                    Instruction::Mul => a.checked_mul(b),
                    Instruction::Div => {
                        if b == 0 {
                            // Restore stack before returning error
                            self.stack.push(a);
                            self.stack.push(b);
                            return Err(RuntimeError::DivideByZero);
                        }
                        a.checked_div(b)
                    }
                    _ => unreachable!(),
                };
                if let Some(res) = result {
                    self.stack.push(res);
                    self.history.push(HistoryEntry {
                        instruction: instruction.clone(),
                        popped_values: vec![b, a],
                        pushed_values: vec![res],
                    });
                    Ok(instruction)
                } else {
                    // Restore stack before returning error
                    self.stack.push(a);
                    self.stack.push(b);
                    Err(RuntimeError::ArithmeticOverflow)
                }
            }
        }
    }

    /// Calls `.forward()` until there are no more instructions or
    /// if there is an error.
    pub fn run(&mut self) -> Result<(), RuntimeError> {
        while !self.instructions.is_empty() {
            self.forward()?;
        }
        Ok(())
    }

    /// *Reverses* the last instruction executed with `.forward()`.
    /// This should undo the last instruction and restore the state of
    /// the stack. Repeated calls should be possible until the stack
    /// is restored to its original state before the first forward call.
    ///
    /// If there is no instruction to reverse, return an error.
    pub fn back(&mut self) -> Result<(), RuntimeError> {
        let history_entry = self.history.pop().ok_or(RuntimeError::NoInstructions)?;

        self.instructions
            .push_front(history_entry.instruction.clone());

        // Reverse the stack changes
        // First, remove the values that were pushed
        // We could also check that the values being popped match the values
        // that were originally pushed, ensuring the state is consistent.
        for _ in &history_entry.pushed_values {
            self.stack.pop().ok_or(RuntimeError::StackUnderflow)?;
        }

        // Then, push back the values that were popped in reverse order
        for &value in history_entry.popped_values.iter().rev() {
            self.stack.push(value);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn instruction_strategy() -> impl Strategy<Value = Instruction> {
        prop_oneof![
            any::<i32>().prop_map(Instruction::Push),
            Just(Instruction::Pop),
            Just(Instruction::Add),
            Just(Instruction::Sub),
            Just(Instruction::Mul),
            Just(Instruction::Div),
        ]
    }

    fn instruction_sequence() -> impl Strategy<Value = Vec<Instruction>> {
        prop::collection::vec(instruction_strategy(), 1..100)
    }

    #[test]
    fn test_property_execution_without_panic() {
        proptest!(|(instructions in instruction_sequence())| {
            let mut interpreter = Interpreter::new();
            interpreter.add_instructions(&instructions);

            let _ = interpreter.run();
            // Ensures no panic occurs during execution.
        });
    }

    #[test]
    fn test_property_reverse_execution() {
        proptest!(|(instructions in instruction_sequence())| {
            let mut interpreter = Interpreter::new();
            interpreter.add_instructions(&instructions);

            let run_result = interpreter.run();

            // Collect executed instructions
            let executed_instructions: Vec<_> = interpreter.history.iter().map(|h| h.instruction.clone()).collect();
            let executed_count = executed_instructions.len();

            if run_result.is_ok() {
                prop_assert_eq!(&executed_instructions, &instructions, "When run is successful executed instructions should be the same as input instructions");
            }

            // Attempt to reverse all executed instructions
            while interpreter.back().is_ok() {}

            prop_assert_eq!(interpreter.stack, vec![], "After reversing the stack should be empty");

            // Compute unexecuted instructions, if any
            let unexecuted_instructions = if run_result.is_err() {
                // If an error occurred, skip the failed instruction
                if executed_count < instructions.len() {
                    &instructions[executed_count + 1..]
                } else {
                    &[]
                }
            } else {
                &[]
            };

            // Build expected instructions after reversal
            let mut expected_instructions = executed_instructions.clone();
            expected_instructions.extend_from_slice(unexecuted_instructions);

            let restored_instructions: Vec<Instruction> = interpreter.instructions.iter().cloned().collect();
            prop_assert_eq!(restored_instructions, expected_instructions);
        });
    }

    #[test]
    fn test_stack_underflow_on_pop() {
        let mut interpreter = Interpreter::new();
        interpreter.add_instructions(&[Instruction::Pop]);
        let result = interpreter.run();
        assert_eq!(result, Err(RuntimeError::StackUnderflow));
        assert!(
            interpreter.instructions.is_empty(),
            "Instruction should have been removed"
        );
    }

    #[test]
    fn test_divide_by_zero() {
        let mut interpreter = Interpreter::new();
        interpreter.add_instructions(&[
            Instruction::Push(10),
            Instruction::Push(0),
            Instruction::Div,
        ]);
        let result = interpreter.run();
        assert_eq!(result, Err(RuntimeError::DivideByZero));
        assert_eq!(interpreter.stack, vec![10, 0], "Stack should be restored");
        assert!(
            interpreter.instructions.is_empty(),
            "Instruction should have been removed"
        );
    }

    #[test]
    fn test_addition_property() {
        proptest!(|(a in any::<i32>(), b in any::<i32>())| {
            let mut interpreter = Interpreter::new();
            interpreter.add_instructions(&[
                Instruction::Push(a),
                Instruction::Push(b),
                Instruction::Add,
            ]);
            let expected_result = match a.checked_add(b) {
                Some(sum) => Ok(sum),
                None => Err(RuntimeError::ArithmeticOverflow),
            };
            let result = interpreter.run();
            match expected_result {
                Ok(sum) => {
                    prop_assert_eq!(result, Ok(()));
                    prop_assert_eq!(interpreter.stack, vec![sum]);
                }
                Err(err) => {
                    prop_assert_eq!(result, Err(err));
                    // Stack should be restored to [a, b]
                    prop_assert_eq!(interpreter.stack, vec![a, b]);
                }
            }
        });
    }

    #[test]
    fn test_multiplication_property() {
        proptest!(|(a in any::<i32>(), b in any::<i32>())| {
            let mut interpreter = Interpreter::new();
            interpreter.add_instructions(&[
                Instruction::Push(a),
                Instruction::Push(b),
                Instruction::Mul,
            ]);
            let expected_result = match a.checked_mul(b) {
                Some(product) => Ok(product),
                None => Err(RuntimeError::ArithmeticOverflow),
            };
            let result = interpreter.run();
            match expected_result {
                Ok(product) => {
                    prop_assert_eq!(result, Ok(()));
                    prop_assert_eq!(interpreter.stack, vec![product]);
                }
                Err(err) => {
                    prop_assert_eq!(result, Err(err));
                    // Stack should be restored to [a, b]
                    prop_assert_eq!(interpreter.stack, vec![a, b]);
                }
            }
        });
    }

    #[test]
    fn test_subtraction_property() {
        proptest!(|(a in any::<i32>(), b in any::<i32>())| {
            let mut interpreter = Interpreter::new();
            interpreter.add_instructions(&[
                Instruction::Push(a),
                Instruction::Push(b),
                Instruction::Sub,
            ]);
            let expected_result = match a.checked_sub(b) {
                Some(diff) => Ok(diff),
                None => Err(RuntimeError::ArithmeticOverflow),
            };
            let result = interpreter.run();
            match expected_result {
                Ok(diff) => {
                    prop_assert_eq!(result, Ok(()));
                    prop_assert_eq!(interpreter.stack, vec![diff]);
                }
                Err(err) => {
                    prop_assert_eq!(result, Err(err));
                    // Stack should be restored to [a, b]
                    prop_assert_eq!(interpreter.stack, vec![a, b]);
                }
            }
        });
    }

    #[test]
    fn test_division_property() {
        proptest!(|(a in any::<i32>(), b in any::<i32>())| {
            let mut interpreter = Interpreter::new();
            interpreter.add_instructions(&[
                Instruction::Push(a),
                Instruction::Push(b),
                Instruction::Div,
            ]);
            let expected_result = a.checked_div(b);
            let result = interpreter.run();
            match expected_result {
                Some(quotient) => {
                    prop_assert_eq!(result, Ok(()));
                    prop_assert_eq!(interpreter.stack, vec![quotient]);
                }
                None => {
                    if b == 0 {
                        prop_assert_eq!(result, Err(RuntimeError::DivideByZero));
                    } else {
                        prop_assert_eq!(result, Err(RuntimeError::ArithmeticOverflow));
                    }
                    // Stack should be restored to [a, b]
                    prop_assert_eq!(interpreter.stack, vec![a, b]);
                }
            }
        });
    }

    #[test]
    fn test_back_with_empty_history() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.back();
        assert_eq!(result, Err(RuntimeError::NoInstructions));
    }

    #[test]
    fn test_forward_with_empty_instructions() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.forward();
        assert_eq!(result, Err(RuntimeError::NoInstructions));
    }

    #[test]
    fn test_run_with_empty_instructions() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.run();
        assert_eq!(result, Ok(()));
        assert!(interpreter.stack.is_empty());
        assert!(interpreter.history.is_empty());
    }

    #[test]
    fn test_history_entry_for_add() {
        let mut interpreter = Interpreter::new();
        interpreter.add_instructions(&[
            Instruction::Push(2),
            Instruction::Push(3),
            Instruction::Add,
        ]);
        interpreter.run().unwrap();
        // Stack should be [5]
        assert_eq!(interpreter.stack, vec![5]);
        // History should have three entries
        assert_eq!(interpreter.history.len(), 3);
        // Check the last history entry
        let last_entry = &interpreter.history[2];
        assert_eq!(last_entry.instruction, Instruction::Add);
        assert_eq!(last_entry.popped_values, vec![3, 2]);
        assert_eq!(last_entry.pushed_values, vec![5]);
    }

    #[test]
    fn test_back_after_error() {
        let mut interpreter = Interpreter::new();
        interpreter.add_instructions(&[
            Instruction::Push(5),
            Instruction::Pop,
            Instruction::Pop, // Will cause StackUnderflow
        ]);
        let result = interpreter.run();
        assert_eq!(result, Err(RuntimeError::StackUnderflow));
        // Perform back operations
        interpreter.back().unwrap(); // Undo Pop
        assert_eq!(interpreter.stack, vec![5]);
        interpreter.back().unwrap(); // Undo Push(5)
        assert_eq!(interpreter.stack, vec![]);
        // No more history
        let result = interpreter.back();
        assert_eq!(result, Err(RuntimeError::NoInstructions));
    }

    #[test]
    fn test_multiple_back_operations() {
        let mut interpreter = Interpreter::new();
        let instructions = [
            Instruction::Push(10),
            Instruction::Push(20),
            Instruction::Add,
            Instruction::Push(5),
            Instruction::Mul,
            Instruction::Push(1),
            Instruction::Pop,
            Instruction::Push(2),
            Instruction::Push(3),
            Instruction::Sub,
            Instruction::Div,
        ];
        interpreter.add_instructions(&instructions);
        interpreter.run().unwrap();
        // Stack should be [150]
        assert_eq!(interpreter.stack, vec![-150]);
        // Perform back operations
        interpreter.back().unwrap(); // Undo Div
        assert_eq!(interpreter.stack, vec![150, -1]);
        interpreter.back().unwrap(); // Undo Sub
        assert_eq!(interpreter.stack, vec![150, 2, 3]);
        interpreter.back().unwrap(); // Undo Push 3
        assert_eq!(interpreter.stack, vec![150, 2]);
        interpreter.back().unwrap(); // Undo Push 2
        assert_eq!(interpreter.stack, vec![150]);
        interpreter.back().unwrap(); // Undo Pop
        assert_eq!(interpreter.stack, vec![150, 1]);
        interpreter.back().unwrap(); // Undo Push 1
        assert_eq!(interpreter.stack, vec![150]);
        interpreter.back().unwrap(); // Undo Mul
        assert_eq!(interpreter.stack, vec![30, 5]);
        interpreter.back().unwrap(); // Undo Push 5
        assert_eq!(interpreter.stack, vec![30]);
        interpreter.back().unwrap(); // Undo Add
        assert_eq!(interpreter.stack, vec![10, 20]);
        interpreter.back().unwrap(); // Undo Push 20
        assert_eq!(interpreter.stack, vec![10]);
        interpreter.back().unwrap(); // Undo Push 10
        assert_eq!(interpreter.stack, vec![]);
        // Attempt to back with empty history
        let result = interpreter.back();
        assert_eq!(result, Err(RuntimeError::NoInstructions));
        // Instructions should be restored
        assert_eq!(interpreter.instructions, VecDeque::from(instructions));
    }

    #[test]
    fn test_history_after_error() {
        let mut interpreter = Interpreter::new();
        interpreter.add_instructions(&[
            Instruction::Push(5),
            Instruction::Pop,
            Instruction::Pop, // This will cause StackUnderflow
        ]);
        let result = interpreter.run();
        assert_eq!(result, Err(RuntimeError::StackUnderflow));
        // History should contain two entries
        assert_eq!(interpreter.history.len(), 2);
        assert_eq!(interpreter.history[0].instruction, Instruction::Push(5));
        assert_eq!(interpreter.history[1].instruction, Instruction::Pop);
        // Instructions queue should be empty
        assert!(interpreter.instructions.is_empty());
    }
}
