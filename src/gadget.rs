use capstone::{Instructions, OwnedInsn, Capstone};
use colored::*;

use crate::err::RVError;
use crate::query;

#[derive (Clone, Copy)]
pub enum OutputMode {
    Inline,
    Block,
}

pub struct Gadget<'a> {
    insns: Vec<OwnedInsn<'a>>,
    hash: u32
}

impl<'a> Gadget<'a> {

    pub fn create(cs: &'a Capstone, insns: Instructions<'a>) -> Result<Self, RVError> {
        let mut g = Gadget {
            insns: insns.iter().map(|x| OwnedInsn::from(x)).collect(),
            hash: 5381,
        };

        for ins in g.insns.iter() {
            for b in ins.bytes() {
                g.hash = g.hash.wrapping_mul(33).wrapping_add(*b as u32);
            }
            if let Ok(_details) = cs.insn_detail(ins) {
                // if let Some(rd) = details.arch_detail().riscv() {
                //     println!("{:?}", ins.id());
                //     for op in rd.operands() {
                //         println!("{:?}", op);
                //     }
                // }
            } else {
                let err = RVError { msg: String::from("Failed to get instruction details") };
                return Err(err);
            }
        }
        return Ok(g);
    }

    pub fn print(&self, mode: OutputMode) {
        match mode {
            OutputMode::Block => self.print_block(),
            OutputMode::Inline => self.print_inline(),
        };
    }

    fn print_block(&self) {
        for (i, ins) in self.insns.iter().enumerate() {
            let branch: bool = i == self.insns.len() - 1;

            let mut insstr = format!("{} {}",
                {
                    let mn = ins.mnemonic().unwrap();
                    if mn.starts_with("c.") {
                        &mn[2..]
                    } else {
                        mn
                    }
                },
                ins.op_str().unwrap(),
            );
            if branch {
                insstr = insstr.red().to_string();
            }
            let bytes = ins.bytes().iter().fold(String::new(), |mut acc, b| {
                acc.push_str(&format!("{:02x} ", b));
                acc
            });
            println!("{:#010x}    {:>015}   {}",
                ins.address(),
                bytes,
                insstr,
            );
        }
    }

    fn print_inline(&self) {
        let mut acc = String::from(format!("{:#010x}     ", self.insns.first().unwrap().address()));
        for (i, ins) in self.insns.iter().enumerate() {
            let branch: bool = i == self.insns.len() - 1;

            let mut insstr = format!("{} {}",
                {
                    let mn = ins.mnemonic().unwrap();
                    if mn.starts_with("c.") {
                        &mn[2..]
                    } else {
                        mn
                    }
                },
                ins.op_str().unwrap(),
            );
            if branch {
                insstr = insstr.red().to_string();
            }
            acc.push_str(&insstr);
            if !branch {
                acc.push_str(if ins.op_str().unwrap().len() == 0 {"; "} else {" ; "});
            }
        }
        println!("{}", acc);
    }

    pub fn satisfies_query(&self, cs: &capstone::Capstone, q: &query::Query) -> bool {
        for ins in self.insns.iter() {
            if let Ok(details) = cs.insn_detail(&ins) {
                if let Some(archdetails) = details.arch_detail().riscv() {
                    if q.is_satisfied(&ins, &details, &archdetails) {
                        return true;
                    }
                }
            }
        }
        return false;
    }

}

impl PartialEq for Gadget<'_> {
    fn eq(&self, other: &Self) -> bool {
        return self.hash == other.hash;
    }
}
