use std::io::Read;
use std::fs::File;
use clap::Parser;
use clap_num::maybe_hex;
use capstone::prelude::*;

/// Compare compiled assembly
#[derive(Parser, Debug)]
#[command(author = None, version = None, about = None, long_about = None)]
struct Args {
    /// Target
    #[arg(short, long)]
    target: String,

    /// Current
    #[arg(short, long)]
    current: String,

    /// Address
    #[arg(short='x', long, value_parser=maybe_hex::<u32>)]
    addr: u32,

    /// Architecture
    #[arg(short, long)]
    arch: capstone::Arch,

    /// Mode
    #[arg(short, long)]
    mode: capstone::Mode,

    /// Endianness
    #[arg(short, long)]
    endianness: Option<capstone::Endian>,
}

const ALIGNMENT: usize = 30;
fn main() {
    let args = Args::parse();

    let cs = Capstone::new_raw(args.arch, args.mode, capstone::NO_EXTRA_MODE, args.endianness).expect("Can't create capstone engine");

    let current = disassemble(&cs, &args.current, args.addr);
    let target = disassemble(&cs, &args.target, args.addr);
    let min_size = std::cmp::min(current.len(), target.len());
    let max_size = std::cmp::max(current.len(), target.len());

    for i in 0..min_size {
        print!("{}{}{}", current[i], " ".repeat(ALIGNMENT - current[i].len()), target[i]);
        if current[i] != target[i] {
            print!("{}<===========", " ".repeat(ALIGNMENT - target[i].len()));
        }
        println!("");
    }

    for i in min_size..max_size {
        if current.len() < min_size {
            print!("{}{}", current[i], " ".repeat(ALIGNMENT - current[i].len()));
        } else {
            print!("{}", " ".repeat(ALIGNMENT));
        }
        if target.len() < min_size {
            print!("{}", target[i]);
        }
        println!("");
    }
}

fn disassemble(cs: &Capstone, file: &str, vram: u32) -> Vec<String> {
    let mut f = File::open(file).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();

    let insns = cs.disasm_all(&buffer, vram as u64).expect("Failed to disassemble");

    let mut insts = vec![];
    for i in insns.as_ref() {
        let op_str = i.op_str().unwrap_or("");
        let inst = i.mnemonic().unwrap();
        insts.push(format!("{inst} {op_str}"));
    }

    insts
}
