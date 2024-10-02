use clap::{Parser, Subcommand};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use crate::errors::RuntimeError;
use crate::interpreter::{Instruction, Interpreter};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a sequence of interpreter commands from a file or standard input
    Script {
        /// File containing interpreter commands. If not provided, reads from standard input.
        #[arg(short, long)]
        file: Option<String>,
    },
    /// Enters interactive shell mode
    Shell,
}

pub fn run_cli() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Script { file } => {
            run_script(file.as_deref());
        }
        Commands::Shell => {
            run_shell();
        }
    }
}

fn run_script(file: Option<&str>) {
    let mut interpreter = Interpreter::new();

    let reader: Box<dyn BufRead> = if let Some(filename) = file {
        let file = File::open(filename).expect("Failed to open file");
        Box::new(BufReader::new(file))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    for line_result in reader.lines() {
        let line = line_result.expect("Failed to read line");
        match parse_and_execute_command(&mut interpreter, &line) {
            Ok(should_continue) => {
                if !should_continue {
                    break;
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                std::process::exit(1); // Exit with non-zero code on error
            }
        }
    }
}

fn run_shell() {
    let mut rl = Editor::<()>::new();
    let mut interpreter = Interpreter::new();

    println!("Reversible Stack-Based Interpreter Shell");
    println!(
        "Enter commands. Type 'help' for a list of commands. Type 'exit' or press Ctrl+D to quit."
    );

    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                match parse_and_execute_command(&mut interpreter, &line) {
                    Ok(should_continue) => {
                        if !should_continue {
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        // Continue the shell session
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Interrupted");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Exiting");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

#[derive(Debug)]
enum Command {
    AddInstruction(Vec<Instruction>),
    CurrentInstruction,
    InstructionQueue,
    Forward,
    Run,
    Back,
    PrintStack,
    Help,
    Exit,
}

fn parse_command(input: &str) -> Result<Command, String> {
    let trimmed_input = input.trim();

    if trimmed_input.is_empty() {
        return Err("Empty command".to_string());
    }

    // Split the input into command and arguments
    let mut parts = trimmed_input.splitn(2, ' ');
    let command_str = parts.next().unwrap().to_lowercase();
    let args = parts.next().unwrap_or("").trim();

    match command_str.as_str() {
        "add" | "add-instruction" => {
            let instructions = parse_instructions_shell(args)?;
            Ok(Command::AddInstruction(instructions))
        }
        "current" | "current-instruction" => Ok(Command::CurrentInstruction),
        "queue" => Ok(Command::InstructionQueue),
        "forward" => Ok(Command::Forward),
        "run" => Ok(Command::Run),
        "back" => Ok(Command::Back),
        "print" | "stack" => Ok(Command::PrintStack),
        "help" => Ok(Command::Help),
        "exit" => Ok(Command::Exit),
        _ => Err(format!("Unknown command: '{}'", command_str)),
    }
}

fn parse_and_execute_command(
    interpreter: &mut Interpreter,
    input: &str,
) -> Result<bool, RuntimeError> {
    match parse_command(input) {
        Ok(Command::Exit) => {
            // Signal to the caller to exit
            Ok(false)
        }
        Ok(command) => execute_command(interpreter, command).map(|_| true),
        Err(err_msg) => {
            println!("{}", err_msg);
            Ok(true)
        }
    }
}

fn execute_command(interpreter: &mut Interpreter, command: Command) -> Result<(), RuntimeError> {
    match command {
        Command::AddInstruction(instructions) => {
            interpreter.add_instructions(&instructions);
            println!("Instructions added.");
            Ok(())
        }
        Command::CurrentInstruction => {
            if let Some(instr) = interpreter.current_instruction() {
                println!("Current instruction: {instr:?}");
            } else {
                println!("No instructions in the queue.");
            }
            Ok(())
        }
        Command::InstructionQueue => {
            println!("Instruction queue: {:?}", interpreter.instructions());
            Ok(())
        }
        Command::Forward => {
            let instruction = interpreter.forward()?;
            println!("Executed {instruction:?}. Stack: {:?}", interpreter.stack());
            Ok(())
        }
        Command::Run => {
            interpreter.run()?;
            println!(
                "All instructions executed. Stack: {:?}",
                interpreter.stack()
            );
            Ok(())
        }
        Command::Back => {
            interpreter.back()?;
            println!(
                "Reversed last instruction. Stack: {:?}",
                interpreter.stack()
            );
            Ok(())
        }
        Command::PrintStack => {
            println!("Stack: {:?}", interpreter.stack());
            Ok(())
        }
        Command::Help => {
            println!("Available commands:");
            println!("  add <instructions>      - Add instructions to the interpreter's queue");
            println!("                           Instructions are separated by semicolons (;)");
            println!("  current                 - Show the current instruction in the queue");
            println!("  queue                   - Show the instruction queue");
            println!("  forward                 - Execute the next instruction");
            println!("  run                     - Execute all instructions");
            println!("  back                    - Reverse the last executed instruction");
            println!("  print                   - Display the current state of the stack");
            println!("  help                    - Display this help message");
            println!("  exit                    - Exit the shell");
            println!("\nInstructions:");
            println!("  PUSH <value>            - Push a value onto the stack");
            println!("  POP                     - Pop a value from the stack");
            println!("  ADD                     - Add the top two values on the stack");
            println!("  SUB                     - Subtract the top two values on the stack");
            println!("  MUL                     - Multiply the top two values on the stack");
            println!("  DIV                     - Divide the top two values on the stack");
            Ok(())
        }
        Command::Exit => {
            // Should not reach here; Exit is handled in parse_and_execute_command
            Ok(())
        }
    }
}

fn parse_instructions_shell(input: &str) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();
    let mut errors = Vec::new();

    for s in input.split(';') {
        let s = s.trim();
        if s.is_empty() {
            continue;
        }
        match parse_instruction(s) {
            Ok(instr) => instructions.push(instr),
            Err(e) => {
                errors.push(format!("Error parsing instruction '{}': {:?}", s, e));
            }
        }
    }

    if !errors.is_empty() {
        for error in errors {
            println!("{}", error);
        }
    }

    if instructions.is_empty() {
        Err("No valid instructions provided".to_string())
    } else {
        Ok(instructions)
    }
}

fn parse_instruction(s: &str) -> Result<Instruction, RuntimeError> {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.is_empty() {
        return Err(RuntimeError::NoInstructions);
    }
    let command = tokens[0].to_uppercase();
    match command.as_str() {
        "PUSH" => {
            if tokens.len() != 2 {
                println!("PUSH requires one argument.");
                return Err(RuntimeError::InvalidCommand);
            }
            let value = tokens[1]
                .parse::<i32>()
                .map_err(|_| RuntimeError::InvalidCommand)?;
            Ok(Instruction::Push(value))
        }
        "POP" => Ok(Instruction::Pop),
        "ADD" => Ok(Instruction::Add),
        "SUB" => Ok(Instruction::Sub),
        "MUL" => Ok(Instruction::Mul),
        "DIV" => Ok(Instruction::Div),
        _ => {
            println!("Invalid instruction: {}", command);
            Err(RuntimeError::InvalidCommand)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::{Instruction, Interpreter};

    #[test]
    fn test_add_and_forward_command() {
        let mut interpreter = Interpreter::new();
        let input = "add PUSH 5; PUSH 3";
        parse_and_execute_command(&mut interpreter, input).unwrap();
        assert_eq!(interpreter.instructions().len(), 2);

        let input = "forward";
        parse_and_execute_command(&mut interpreter, input).unwrap();
        assert_eq!(*interpreter.stack(), vec![5]);
        assert_eq!(interpreter.instructions().len(), 1);

        let input = "forward";
        parse_and_execute_command(&mut interpreter, input).unwrap();
        assert_eq!(*interpreter.stack(), vec![5, 3]);
        assert_eq!(interpreter.instructions().len(), 0);
    }

    #[test]
    fn test_back_command() {
        let mut interpreter = Interpreter::new();

        // Add instructions
        let input = "add PUSH 5; PUSH 3";
        parse_and_execute_command(&mut interpreter, input).unwrap();

        // Run the instructions
        let input = "run";
        parse_and_execute_command(&mut interpreter, input).unwrap();

        // Now the stack should be [5, 3]
        assert_eq!(*interpreter.stack(), vec![5, 3]);

        // Call back
        let input = "back";
        parse_and_execute_command(&mut interpreter, input).unwrap();

        // Stack should now be [5]
        assert_eq!(*interpreter.stack(), vec![5]);

        // The instruction should be back in the interpreter's instructions
        assert_eq!(interpreter.instructions().len(), 1);
        assert_eq!(
            *interpreter.current_instruction().unwrap(),
            Instruction::Push(3)
        );
    }

    #[test]
    fn test_unknown_command() {
        let mut interpreter = Interpreter::new();
        let input = "unknown";
        parse_and_execute_command(&mut interpreter, input).unwrap();
        // Should print "Unknown command: 'unknown'" and continue
    }

    #[test]
    fn test_parse_instruction_invalid() {
        let result = parse_instruction("INVALID");
        assert_eq!(result.unwrap_err(), RuntimeError::InvalidCommand);
    }
}
