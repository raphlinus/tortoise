
use rspirv::binary::{Parser};

mod ctx;

use ctx::Ctx;

fn main() {
    let filename = std::env::args().skip(1).next().expect("need spv file name");
    let bytes = std::fs::read(filename).unwrap();
    let mut ctx = Ctx::default();
    let parser = Parser::new(&bytes, &mut ctx);
    parser.parse().unwrap();
    println!("{:?}", ctx.names);
    for func in &ctx.funcs {
        println!("{}", ctx.function_rs(func));
    }
}
