use std::collections::HashMap;

use rspirv::binary::{Consumer, ParseAction};
use rspirv::dr::{Instruction, ModuleHeader, Operand};
use rspirv::spirv::{Op, StorageClass, Word};

#[derive(Default)]
pub struct Ctx {
    pub names: HashMap<Word, String>,
    member_names: HashMap<(Word, Word), String>,
    inst_map: HashMap<Word, Instruction>,
    pub funcs: Vec<Function>,
    cur_func: Option<Function>,
}

#[derive(Debug)]
pub struct Function {
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
        //println!("inst: {:?}", inst);
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
                Op::FunctionParameter => (), // TODO
                _ => func.blocks.last_mut().unwrap().insts.push(inst.clone()),
            }
        } else {
            match inst.class.opcode {
                Op::Name => {
                    if let (Operand::IdRef(id), Operand::LiteralString(s)) =
                        (&inst.operands[0], &inst.operands[1])
                    {
                        self.names.insert(*id, s.clone());
                    }
                }
                Op::MemberName => {
                    if let (
                        Operand::IdRef(id),
                        Operand::LiteralInt32(ix),
                        Operand::LiteralString(s),
                    ) = (&inst.operands[0], &inst.operands[1], &inst.operands[2])
                    {
                        self.member_names.insert((*id, *ix), s.clone());
                    }
                }
                Op::TypeStruct => {
                    let struct_id = inst.result_id.unwrap();
                    println!("struct {} {{", self.name(struct_id));
                    for (i, ty) in inst.operands.iter().enumerate() {
                        if let Some(member_name) = self.member_names.get(&(struct_id, i as Word)) {
                            if let Operand::IdRef(ty_id) = ty {
                                println!("    {}: {},", member_name, self.type_rs(*ty_id));
                            }
                        }
                    }
                    println!("}}");
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

    fn lvalue_rs(&self, operand: &Operand) -> String {
        match operand {
            Operand::IdRef(id) => {
                if let Some(result) = self.opt_rvalue_rs(*id) {
                    return result;
                }
                if let Some(inst) = self.inst_map.get(&id) {
                    match inst.class.opcode {
                        Op::Constant => {
                            return self.constant(inst.result_type.unwrap(), &inst.operands[0])
                        }
                        _ => (),
                    }
                }
                // We're hoping some statement bound this value.
                self.name(*id)
            }
            _ => format!("[unhandled {:?}]", operand),
        }
    }

    fn opt_rvalue_rs(&self, id: Word) -> Option<String> {
        if let Some(inst) = self.inst_map.get(&id) {
            match inst.class.opcode {
                Op::Variable => Some(self.name(inst.result_id.unwrap())),
                _ => None,
            }
        } else {
            None
        }
    }

    fn rvalue_rs(&self, operand: &Operand) -> String {
        match operand {
            Operand::IdRef(id) => {
                if let Some(result) = self.opt_rvalue_rs(*id) {
                    return result;
                }
                if let Some(inst) = self.inst_map.get(&id) {
                    match inst.class.opcode {
                        _ => format!("[unhandled {:?}]", inst.class.opcode),
                    }
                } else {
                    panic!("no inst with result id {}", id)
                }
            }
            _ => format!("[unhandled {:?}]", operand),
        }
    }

    fn type_rs(&self, id: Word) -> String {
        if let Some(inst) = self.inst_map.get(&id) {
            match inst.class.opcode {
                Op::TypeInt => {
                    if let (Operand::LiteralInt32(size), Operand::LiteralInt32(sign)) =
                        (&inst.operands[0], &inst.operands[1])
                    {
                        if *sign == 0 {
                            format!("u{}", size)
                        } else {
                            format!("i{}", size)
                        }
                    } else {
                        panic!("unknown TypeInt args: {:?}", inst.operands);
                    }
                }
                Op::TypeRuntimeArray => {
                    if let Operand::IdRef(inner) = inst.operands[0] {
                        format!("*mut {}", self.type_rs(inner))
                    } else {
                        panic!("unknown TypeRuntimeArray args: {:?}", inst.operands);
                    }
                }
                _ => format!("[unhandled type {:?}]", inst.class.opcode),
            }
        } else {
            panic!("no inst for type {}", id);
        }
    }

    fn deref_ptr(&self, id: Word) -> Word {
        if let Some(inst) = self.inst_map.get(&id) {
            match inst.class.opcode {
                Op::TypePointer => match inst.operands[1] {
                    Operand::IdRef(id) => id,
                    _ => panic!("don't know how to deref non-id"),
                },
                _ => panic!("dereferencing non-points"),
            }
        } else {
            panic!("No inst for ptr {}", id);
        }
    }

    fn constant(&self, ty: Word, operand: &Operand) -> String {
        if let Some(ty_inst) = self.inst_map.get(&ty) {
            match (ty_inst.class.opcode, operand) {
                (Op::TypeInt, Operand::LiteralInt32(i)) => {
                    if let (Operand::LiteralInt32(size), Operand::LiteralInt32(sign)) =
                        (&ty_inst.operands[0], &ty_inst.operands[1])
                    {
                        if *sign == 0 {
                            format!("{}u{}", i, size)
                        } else {
                            format!("{}i{}", (*i as i32), size)
                        }
                    } else {
                        panic!("unknown TypeInt args: {:?}", ty_inst.operands);
                    }
                }
                _ => panic!("unknown constant type"),
            }
        } else {
            panic!("No inst for constant type {}", ty);
        }
    }

    fn binop(&self, inst: &Instruction, op: &str) -> Option<String> {
        let operand0 = self.lvalue_rs(&inst.operands[0]);
        let operand1 = self.lvalue_rs(&inst.operands[1]);
        let dst = self.name(inst.result_id.unwrap());
        Some(format!("let {} = {} {} {};", dst, operand0, op, operand1))
    }

    fn stmt_rs(&self, inst: &Instruction) -> Option<String> {
        match inst.class.opcode {
            Op::Load => {
                let src = self.lvalue_rs(&inst.operands[0]);
                let dst = self.name(inst.result_id.unwrap());
                Some(format!("let {} = {};", dst, src))
            }
            Op::Store => {
                let src = self.lvalue_rs(&inst.operands[1]);
                let dst = self.rvalue_rs(&inst.operands[0]);
                Some(format!("{} = {};", dst, src))
            }
            // These ops don't generate statements, just build values.
            Op::AccessChain => None,
            // pretty good chance this should be wrapping_add, but let's keep things simple.
            Op::IAdd => self.binop(inst, "+"),
            Op::Variable => {
                assert_eq!(
                    inst.operands[0],
                    Operand::StorageClass(StorageClass::Function)
                );
                let var_name = self.name(inst.result_id.unwrap());
                let ty = self.deref_ptr(inst.result_type.unwrap());
                Some(format!("let mut {}: {};", var_name, self.type_rs(ty)))
            }
            _ => Some(format!("// unhandled inst {:?}", inst.class.opcode)),
        }
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

    pub fn function_rs(&self, function: &Function) -> String {
        let mut buf = format!("function {}:\n", self.name(function.result_id));
        for block in &function.blocks {
            buf.push_str(&self.basic_block_rs(block));
        }
        buf
    }
}
