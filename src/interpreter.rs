use crate::errors::RuntimeError;
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Push(i32),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
struct HistoryEntry {
    instruction: Instruction,
    popped_values: Vec<i32>,
    pushed_values: Vec<i32>,
}

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

    pub fn instructions(&self) -> &VecDeque<Instruction> {
        &self.instructions
    }

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
            let initial_stack = Vec::new();

            let mut interpreter = Interpreter::new();
            interpreter.add_instructions(&instructions);

            let run_result = interpreter.run();

            // Collect executed instructions before reversing
            let executed_instructions: Vec<_> = interpreter.history.iter().map(|h| h.instruction.clone()).collect();

            while interpreter.back().is_ok() {}

            prop_assert_eq!(interpreter.stack, initial_stack);

            let executed_count = executed_instructions.len();

            let failed_instruction_index = executed_count;

            let unexecuted_instructions = if run_result.is_err() {
                // Skip the failed instruction
                if failed_instruction_index < instructions.len() {
                    &instructions[failed_instruction_index + 1..]
                } else {
                    &[]
                }
            } else {
                &instructions[executed_count..]
            };

            // Build expected instructions after reversal
            let mut expected_instructions_after_reversal = executed_instructions.clone();
            expected_instructions_after_reversal.extend_from_slice(unexecuted_instructions);

            let restored_instructions: Vec<Instruction> = interpreter.instructions.iter().cloned().collect();

            prop_assert_eq!(restored_instructions, expected_instructions_after_reversal);
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

    #[test]
    fn test_interpreter_consistency() {
        proptest!(|(instructions in instruction_sequence())| {
            let mut interpreter = Interpreter::new();
            interpreter.add_instructions(&instructions);

            // Keep a clone of the initial instructions for later comparison
            let initial_instructions = instructions.clone();

            let run_result = interpreter.run();

            // Collect executed instructions
            let executed_instructions: Vec<_> = interpreter.history.iter().map(|h| h.instruction.clone()).collect();

            // Attempt to reverse all executed instructions
            while interpreter.back().is_ok() {}

            // After reversing, the stack should be empty
            prop_assert_eq!(interpreter.stack, vec![]);

            let executed_count = executed_instructions.len();

            // Determine the index of the failed instruction, if any
            let failed_index = executed_count;

            // Compute unexecuted instructions
            let unexecuted_instructions = if run_result.is_err() {
                // If an error occurred, skip the failed instruction
                if failed_index < initial_instructions.len() {
                    &initial_instructions[failed_index + 1..]
                } else {
                    &[]
                }
            } else {
                &initial_instructions[executed_count..]
            };

            // Build expected instructions after reversal
            let mut expected_instructions = executed_instructions.clone();
            expected_instructions.extend_from_slice(unexecuted_instructions);

            let restored_instructions: Vec<Instruction> = interpreter.instructions.iter().cloned().collect();
            prop_assert_eq!(restored_instructions, expected_instructions);
        });
    }
}
