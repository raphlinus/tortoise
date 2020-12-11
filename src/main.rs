
use std::collections::HashMap;

use rspirv::binary::{Consumer, ParseAction, Parser};
use rspirv::dr::{Instruction, ModuleHeader, Operand};
use rspirv::spirv::{Op, Word};

#[derive(Default)]
struct MyConsumer {
    names: HashMap<Word, String>,
}

impl Consumer for MyConsumer {
    fn initialize(&mut self) -> ParseAction {
        ParseAction::Continue
    }

    fn finalize(&mut self) -> ParseAction {
        ParseAction::Continue
    }

    fn consume_header(&mut self, module: ModuleHeader) -> ParseAction {
        println!("header: {:?}", module);
        ParseAction::Continue
    }

    fn consume_instruction(&mut self, inst: Instruction) -> ParseAction {
        println!("inst: {:?}", inst);
        match inst.class.opcode {
            Op::Name => {
                println!("operands: {:?}", inst.operands);
                if let (Operand::IdRef(id), Operand::LiteralString(s)) = (&inst.operands[0], &inst.operands[1]) {
                    self.names.insert(*id, s.clone());
                }
            }
            _ => (),
        }
        ParseAction::Continue
    }
}

fn main() {
    let filename = std::env::args().skip(1).next().expect("need spv file name");
    let bytes = std::fs::read(filename).unwrap();
    let mut consumer = MyConsumer::default();
    let parser = Parser::new(&bytes, &mut consumer);
    parser.parse().unwrap();
    println!("{:?}", consumer.names);
}
