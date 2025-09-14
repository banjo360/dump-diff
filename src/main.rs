use capstone::prelude::*;
use clap::Parser;
use clap_num::maybe_hex;
use fastxfix::CommonStr;
use similar::utils::diff_lines;
use similar::Algorithm;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

/// Compare compiled assembly
#[derive(Parser, Debug)]
#[command(author = None, version = None, about = None, long_about = None)]
struct Args {
    /// Filename + optional offset
    #[arg(short, long)]
    target: String,

    /// Filename + optional offset
    #[arg(short, long)]
    current: String,

    /// Virtual address
    #[arg(short='x', long, value_parser=maybe_hex::<u32>)]
    addr: u32,

    /// Number of bytes to compare (default: all)
    #[arg(short, long)]
    length: Option<u64>,

    /// Architecture
    #[arg(short, long)]
    arch: capstone::Arch,

    /// Mode
    #[arg(short, long)]
    mode: capstone::Mode,

    /// Endianness (default: little)
    #[arg(short, long)]
    endianness: Option<capstone::Endian>,
}

fn main() {
    let args = Args::parse();

    let cs = Capstone::new_raw(
        args.arch,
        args.mode,
        capstone::NO_EXTRA_MODE,
        args.endianness,
    )
    .expect("Can't create capstone engine");
    let (curr_file, curr_offset) = extract_offset(&args.current);
    let (targ_file, targ_offset) = extract_offset(&args.target);

    let current = disassemble(&cs, curr_file, curr_offset, args.length, args.addr);
    let target = disassemble(&cs, targ_file, targ_offset, args.length, args.addr);

    let l_align = current.iter().map(String::len).max().unwrap() + 0;
    let r_align = current.iter().map(String::len).max().unwrap() + 1;

    let cs = current.join("\n");
    let ts = target.join("\n");

    let differ = diff_lines(Algorithm::Myers, &cs, &ts);

    let mut current = vec![];
    let mut target = vec![];

    let mut current_delete = vec![];
    let mut target_add = vec![];
    for (op, s) in differ {
        let string = s.trim();
        match op {
            similar::ChangeTag::Equal => {
                if current_delete.len() == target_add.len() {
                    current.append(&mut current_delete);
                    target.append(&mut target_add);
                } else {
                    if current_delete.len() == 0 {
                        current.append(&mut vec![""; target_add.len()]);
                        target.append(&mut target_add);
                    } else if target_add.len() == 0 {
                        target.append(&mut vec![""; current_delete.len()]);
                        current.append(&mut current_delete);
                    } else {
                        let (mut c, mut t) = synchronise(&current_delete, &target_add);

                        current.append(&mut c);
                        target.append(&mut t);
                    }
                }

                target_add.clear();
                current_delete.clear();

                current.push(string);
                target.push(string);
            }
            similar::ChangeTag::Delete => {
                current_delete.push(string);
            }
            similar::ChangeTag::Insert => {
                target_add.push(string);
            }
        }
    }

    if current_delete.len() > 0 || target_add.len() > 0 {
        let (mut c, mut t) = synchronise(&current_delete, &target_add);

        current.append(&mut c);
        target.append(&mut t);
    }

    assert_eq!(current.len(), target.len());

    println!(
        "current:{} | target:",
        " ".repeat(l_align - "current:".len())
    );
    for i in 0..current.len() {
        print!(
            "{}{} | {}",
            current[i],
            " ".repeat(l_align - current[i].len()),
            target[i]
        );
        if current[i] != target[i] {
            print!("{}<===========", " ".repeat(r_align - target[i].len()));
        }
        println!("");
    }
}

fn synchronise<'a>(left: &Vec<&'a str>, right: &Vec<&'a str>) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut l = vec![];
    let mut r = vec![];

    let mut il = 0;
    let mut ir = 0;

    while il < left.len() && ir < right.len() {
        let oil = il;
        let oir = ir;
        let ret1 = left.iter().skip(il).position(|lp| {
            let strings = vec![lp.to_string(), right[ir].to_string()];

            if let Some(prefix) = strings.common_prefix() {
                prefix.len() > 2
            } else {
                false
            }
        });
        let ret2 = right.iter().skip(ir).position(|rp| {
            let strings = vec![rp.to_string(), left[il].to_string()];

            if let Some(prefix) = strings.common_prefix() {
                prefix.len() > 2
            } else {
                false
            }
        });

        if let (Some(r1), Some(r2)) = (ret1, ret2) {
            assert_eq!(r1, r2);

            if r1 == 0 {
                l.push(left[il]);
                r.push(right[ir]);
                il += 1;
                ir += 1;
            } else {
                r.push("");
                l.push(left[il]);
                il += 1;
            }
        } else {
            if ret1.is_none() && ret2.is_some() {
                let Some(r2) = ret2 else {
                    todo!();
                };

                let r2 = r2 + ir;

                while ir < r2 {
                    l.push("");
                    r.push(right[ir]);
                    ir += 1;
                }
            } else if ret2.is_none() && ret1.is_some() {
                let Some(r1) = ret1 else {
                    todo!();
                };

                let r1 = r1 + il;

                while il < r1 {
                    r.push("");
                    l.push(left[il]);
                    il += 1;
                }
            } else {
                l.push(left[il]);
                r.push(right[ir]);
                il += 1;
                ir += 1;
            }
        }

        if il == oil && ir == oir {
            todo!();
        }
    }

    while il < left.len() {
        r.push("");
        l.push(left[il]);
        il += 1;
    }
    while ir < right.len() {
        l.push("");
        r.push(right[ir]);
        ir += 1;
    }

    assert_eq!(l.len(), r.len());

    (l, r)
}

fn extract_offset(filename: &str) -> (&str, u64) {
    if filename.contains(':') {
        let mut parts = filename.split(':');
        let filename = parts.next().unwrap();
        let offset = clap_num::maybe_hex::<u64>(parts.next().unwrap()).unwrap();
        (filename, offset)
    } else {
        (filename, 0)
    }
}

fn disassemble(cs: &Capstone, file: &str, offset: u64, len: Option<u64>, vram: u32) -> Vec<String> {
    let mut f = File::open(file).unwrap();
    f.seek(SeekFrom::Start(offset)).unwrap();

    let mut buffer = Vec::new();
    if let Some(len) = len {
        let mut handle = f.take(len);
        handle.read_to_end(&mut buffer).unwrap();
    } else {
        f.read_to_end(&mut buffer).unwrap();
    };
    assert!(buffer.len() != 0);

    let mut insts = vec![];
    for (pos, i) in buffer.chunks_exact(4).enumerate() {
        let insns = cs
            .disasm_count(i, (vram as u64) + (pos as u64), 1)
            .expect("Failed to disassemble");
        if insns.len() == 1 {
            for i in insns.as_ref() {
                let op_str = i.op_str().unwrap_or("");
                let inst = i.mnemonic().unwrap();
                insts.push(format!("{inst} {op_str}"));
            }
        } else if insns.len() == 0 {
            assert_eq!(i, [0, 0, 0, 0]);
            insts.push("0x00000000".into());
        } else {
            unreachable!();
        }
    }

    insts
}
