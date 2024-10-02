#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    DivideByZero,
    StackUnderflow,
    NoInstructions,
    ArithmeticOverflow,
    InvalidCommand,
}
