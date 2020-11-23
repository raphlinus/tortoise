use rspirv::binary::{Consumer, ParseAction, Parser};
use rspirv::dr::{Instruction, ModuleHeader};

struct MyConsumer;

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
        ParseAction::Continue
    }
}

fn main() {
    let filename = std::env::args().skip(1).next().expect("need spv file name");
    let bytes = std::fs::read(filename).unwrap();
    let mut consumer = MyConsumer;
    let parser = Parser::new(&bytes, &mut consumer);
    parser.parse().unwrap();
}
