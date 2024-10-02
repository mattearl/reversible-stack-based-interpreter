# Reversible stack-based interpreter

1. The interpreter uses a stack to store values (all i32).
2. Instructions are stored in a VecDeque.
3. The main operations are PUSH, POP, ADD, MUL, SUB, and DIV.
4. The interpreter should be able to execute instructions forward and backward (undo).
5. Error handling for various scenarios is required.

## Implementation

There is a basic harness included in `./src`. Feel free to throw it out if you don't like it.
You are free to choose whicever data structures at your own discretion.

1. Implement the `Interpreter` struct:
   - Use a `VecDeque<any>` for instructions
   - Use a `Vec<i32>` for the stack
   - Add fields to support the `.back()` method (history)

2. Implement the basic methods:
   - `new()`: Create a new interpreter with empty stack and instructions
   - `add_instructions()`: Add instructions to the end of the queue
   - `current_instruction()`: Return a mutable reference to the next instruction

3. Implement the `forward()` method:
   - Parse and execute the first instruction in the queue
   - Handle all possible errors
   - Store information for the `back()` method

4. Implement the `run()` method:
   - Execute `forward()` until all instructions are processed or an error occurs

5. Implement the `back()` method:
   - Reverse the last executed instruction
   - Restore both the instruction queue and stack state
   - Handle the case when there are no instructions to reverse

6. Error handling:
   - Use the provided `RuntimeError` enum for various error cases

7. Parsing:
   - Parse instructions and arguments (assume single space separation)
   - Convert string arguments to i32 where necessary

8. Testing:
   - Ensure all public fields (instructions and stack) are accessible for testing
   - Implement comprehensive tests for all operations and edge cases

Remember to restore both the instruction queue and stack state when reversing operations.
