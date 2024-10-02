# Reversible Stack-Based Interpreter

## Overview

This project implements a **reversible stack-based interpreter** with support for both **script execution** and an
**interactive shell mode**. The interpreter processes a set of basic stack-manipulation and arithmetic instructions,
offering forward execution as well as the ability to reverse previous operations, making it highly versatile for
debugging and experimenting with stack-based logic.

## Features

- **Instructions**: Supports fundamental stack-based operations:
  - `PUSH <value>`: Push an integer onto the stack.
  - `POP`: Remove the top value from the stack.
  - `ADD`, `SUB`, `MUL`, `DIV`: Perform arithmetic operations on the top two stack values.
- **Reversible Execution**: Every instruction is logged in history, allowing you to undo operations step-by-step.
- **CLI Modes**:
  - **Script Mode**: Execute a series of commands from a file or standard input.
  - **Interactive Shell Mode**: A command-line interface where users can interactively add and execute commands, view
      the stack, and reverse operations.

## Error Handling

The interpreter provides detailed runtime error handling to deal with common issues like:
- **Divide by Zero**: An error is raised when attempting to divide by zero.
- **Stack Underflow**: Attempting to pop or operate on an empty or insufficiently populated stack.
- **No Instructions**: No instructions available for execution.
- **Arithmetic Overflow**: Operations that result in numeric overflow.
- **Invalid Commands**: Unrecognized or malformed commands.

## Command-Line Interface

The CLI provides two main modes:
1. **Script Mode**: Run a series of interpreter commands from a file or standard input. To execute in this mode:
   ```sh
   ./reversible_interpreter script --file <path-to-script>
   ```
   If no file is provided, the program reads commands from standard input.

2. **Shell Mode**: Enter an interactive session where you can type commands and see results immediately:
   ```sh
   ./reversible_interpreter shell
   ```

## Getting Started

To build and run the interpreter, follow these steps:

1. **Install Dependencies**: Make sure you have `rustc` and `cargo` installed.
2. **Build**: Run the following command in the project root:
   ```sh
   cargo build
   ```
3. **Run the CLI**: You can either provide a script or use the interactive shell:
   ```sh
   cargo run -- script --file example.txt
   cargo run -- shell
   ```

## Interactive Shell Commands

In **Shell Mode**, you can interact with the interpreter by entering commands directly. Here’s a list of available commands you can use:

```
Available commands:
  add <instructions>      - Add instructions to the interpreter's queue
                           Instructions are separated by semicolons (;)
  current                 - Show the current instruction in the queue
  queue                   - Show the instruction queue
  forward                 - Execute the next instruction
  run                     - Execute all instructions
  back                    - Reverse the last executed instruction
  print                   - Display the current state of the stack
  help                    - Display this help message
  exit                    - Exit the shell

Instructions:
  PUSH <value>            - Push a value onto the stack
  POP                     - Pop a value from the stack
  ADD                     - Add the top two values on the stack
  SUB                     - Subtract the top two values on the stack
  MUL                     - Multiply the top two values on the stack
  DIV                     - Divide the top two values on the stack
```

## Example Usage: Shell Mode

When in **Shell Mode**, you can use the above commands to interact with the interpreter. For example:

```
> add PUSH 5; PUSH 10; ADD
Instructions added.
> forward
Executed Push(5). Stack: [5]
> forward
Executed Push(10). Stack: [5, 10]
> forward
Executed Add. Stack: [15]
> back
Reversed last instruction. Stack: [5, 10]
> 
```

To exit the shell, type `exit` or press `Ctrl+D`.

### Example Usage: Script Mode

In **Script Mode**, you can provide a file containing a sequence of commands to be executed by the interpreter. Here's an example demonstrating how to create and run a script:

1. **Create a Script File**  
   Create a file (e.g., `example.txt`) containing interpreter commands:

   ```sh
   $ cat example.txt
   add PUSH 10; PUSH 20; ADD; PUSH 5; MUL; PUSH 1; POP; PUSH 2; PUSH 3; SUB; DIV
   run
   back
   back
   back
   back
   back
   back
   back
   back
   back
   back
   ```

2. **Run the Script**  
   Use the following command to run the script using the interpreter:

   ```sh
   $ cargo run -- script --file example.txt
   ```

3. **Script Output**  
   The interpreter processes each command in the script file sequentially. Here's the output of the script:

   ```
   Instructions added.
   All instructions executed. Stack: [-150]
   Reversed last instruction. Stack: [150, -1]
   Reversed last instruction. Stack: [150, 2, 3]
   Reversed last instruction. Stack: [150, 2]
   Reversed last instruction. Stack: [150]
   Reversed last instruction. Stack: [150, 1]
   Reversed last instruction. Stack: [150]
   Reversed last instruction. Stack: [30, 5]
   Reversed last instruction. Stack: [30]
   Reversed last instruction. Stack: [10, 20]
   Reversed last instruction. Stack: [10]
   Reversed last instruction. Stack: []
   ```


