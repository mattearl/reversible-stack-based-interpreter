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
