
use std::collections::HashMap;

use rspirv::binary::{Consumer, ParseAction, Parser};
use rspirv::dr::{Instruction, ModuleHeader, Operand};
use rspirv::spirv::{Op, Word};

#[derive(Default)]
struct Ctx {
    names: HashMap<Word, String>,
    inst_map: HashMap<Word, Instruction>,
    funcs: Vec<Function>,
    cur_func: Option<Function>,
}

#[derive(Debug)]
struct Function {
    result_id: Word,
    blocks: Vec<BasicBlock>,
}

#[derive(Debug)]
struct BasicBlock {
    result_id: Word,
    insts: Vec<Instruction>,
}

impl Consumer for Ctx {
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
        if inst.class.opcode == Op::FunctionEnd {
            self.funcs.push(self.cur_func.take().unwrap());
        } else if let Some(func) = self.cur_func.as_mut() {
            match inst.class.opcode {
                Op::Label => {
                    let block = BasicBlock {
                        result_id: inst.result_id.unwrap(),
                        insts: Vec::new(),
                    };
                    func.blocks.push(block);
                }
                _ => func.blocks.last_mut().unwrap().insts.push(inst.clone()),
            }
        } else {
            match inst.class.opcode {
                Op::Name => {
                    println!("operands: {:?}", inst.operands);
                    if let (Operand::IdRef(id), Operand::LiteralString(s)) = (&inst.operands[0], &inst.operands[1]) {
                        self.names.insert(*id, s.clone());
                    }
                }
                Op::Function => {
                    let function = Function {
                        result_id: inst.result_id.unwrap(),
                        blocks: Vec::new(),
                    };
                    self.cur_func = Some(function);
                }
                _ => (),
            }
        }
        if let Some(id) = inst.result_id {
            self.inst_map.insert(id, inst);
        }
        ParseAction::Continue
    }
}

impl Ctx {
    fn name(&self, id: Word) -> String {
        if let Some(name) = self.names.get(&id) {
            name.clone()
        } else {
            format!("id_{}", id)
        }
    }

    fn lvalue_rs(&self, id: Word) -> String {
        if let Some(inst) = self.inst_map.get(&id) {
            match inst.class.opcode {
                _ => format!("[unhandled {:?}]", inst.class.opcode)
            }
        } else {
            panic!("no inst with result id {}", id)
        }
    }

    fn rvalue_rs(&self, id: Word) -> String {
        if let Some(inst) = self.inst_map.get(&id) {
            match inst.class.opcode {
                _ => format!("[unhandled {:?}]", inst.class.opcode)
            }
        } else {
            panic!("no inst with result id {}", id)
        }
    }

    fn stmt_rs(&self, inst: &Instruction) -> Option<String> {
        Some(format!("// unhandled inst {:?}", inst.class.opcode))
    }

    fn basic_block_rs(&self, block: &BasicBlock) -> String {
        let mut buf = format!("// block {}\n", block.result_id);
        for inst in &block.insts {
            if let Some(stmt) = self.stmt_rs(inst) {
                buf.push_str(&stmt);
                buf.push('\n');
            }
        }
        buf
    }

    fn function_rs(&self, function: &Function) -> String {
        let mut buf = format!("function {}:\n", self.name(function.result_id));
        for block in &function.blocks {
            buf.push_str(&self.basic_block_rs(block));
        }
        buf
    }
}

fn main() {
    let filename = std::env::args().skip(1).next().expect("need spv file name");
    let bytes = std::fs::read(filename).unwrap();
    let mut consumer = Ctx::default();
    let parser = Parser::new(&bytes, &mut consumer);
    parser.parse().unwrap();
    println!("{:?}", consumer.names);
    for func in &consumer.funcs {
        println!("{}", consumer.function_rs(func));
    }
}
