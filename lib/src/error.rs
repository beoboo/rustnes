use std::{error, fmt};

use crate::instructions::addressing_mode::AddressingMode;

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    Assembler(String),
    Lexer(String),
    Parser(String),
    UndefinedStage(String),
    UnknownOpCode(String),
    UnknownAddressingMode(String, AddressingMode),
    UnterminatedInput,
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Error::Assembler(message) => write!(f, "{}", message),
            Error::Lexer(message) => write!(f, "{}", message),
            Error::Parser(message) => write!(f, "{}", message),
            Error::UndefinedStage(stage) => write!(f, "Undefined stage: '{}'", stage),
            Error::UnknownAddressingMode(op_code, addressing_mode) => write!(f, "Unmapped addressing mode '{:?}' for op code: {}", addressing_mode, op_code),
            Error::UnknownOpCode(op_code) => write!(f, "Unknown op code: {}", op_code),
            Error::UnterminatedInput => write!(f, "EOF"),
        }
    }
}

impl error::Error for Error {}

pub fn report_stage_error<S: Into<String>, T>(err: S, stage: &str) -> Result<T, Error> {
    let error = err.into();
    if &error == "EOF" {
        return Err(Error::UnterminatedInput);
    }

    match stage {
        "assembler" => Err(Error::Assembler(error)),
        "lexer" => Err(Error::Lexer(error)),
        "parser" => Err(Error::Parser(error)),
        _ => Err(Error::UndefinedStage(stage.into()))
    }
}

