use crate::assembler::Assembler;
use crate::disassembler::line::Line;
use crate::parser::Parser;
use crate::types::Byte;

mod line;

#[derive(Debug, Default)]
pub struct Disassembler<'a> {
    instructions: Vec<Line<'a>>
}

impl Disassembler<'_> {
    pub fn disassemble(&mut self, source: &[Byte]) {
        // let mut instructions = vec![];

        // let it = source.iter();
        //
        // match it {
        //
        // }
    }
}

#[cfg(test)]
mod tests {
    // use hamcrest2::core::*;
    use hamcrest2::prelude::*;

    use super::*;

    #[test]
    fn simple() {
        let mut disassembler = Disassembler::default();
        disassembler.disassemble(build_program("LDA #1").as_slice());

        assert_that!(disassembler.instructions[0], eq(Line::new(0x0000, "LDA #1")));
    }
}

fn build_program(source: &str) -> Vec<Byte> {
    println!("Processing:\n {}", source);
    let assembler = Assembler::default();
    let parser = Parser::default();
    let tokens = parser.parse(source).unwrap();
    // println!("Tokens: {:?}", tokens);

    let program = match assembler.assemble(tokens) {
        Ok(program) => program,
        Err(e) => panic!("Assembler error: {}", e)
    };
    println!("Program: {:x?}", program);

    program.data
}
