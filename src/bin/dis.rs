/*!
 * Dissasmble that thing
 *
 * Author: Dave Eddy <ysap@daveeddy.com>
 * Date: December 21, 2025
 * License: MIT
 */

use log::{debug, info, trace};
use std::env;
use std::fs;
use std::io::{self, Read, Write};

#[derive(Default)]
struct VM {
    ram: Vec<u8>,
    registers: [u16; 8],
    addr: u16, // addr pointer
    stack: Vec<u16>,
    running: bool,
    level: usize,
    pub input_buffer: Vec<u8>,
}

enum ValueType {
    Register(u16),
    Literal(u16),
}

#[allow(dead_code)]
impl VM {
    fn new(rom: Vec<u8>) -> Self {
        Self { ram: rom, running: true, ..Default::default() }
    }

    fn get_ram(&self, addr: u16) -> u16 {
        let ptr = (addr * 2) as usize;
        let low = self.ram[ptr] as u16;
        let high = self.ram[ptr + 1] as u16;

        let num = (high << 8) + low;
        trace!(
            "self.get_ram: ptr={} (low={} high={}) num={}",
            ptr, low, high, num
        );

        num
    }

    // get the raw number from the rom
    fn get_ram_value(&self, addr: u16) -> ValueType {
        let num = self.get_ram(addr);

        if num < 32768 {
            // it's a literal value
            ValueType::Literal(num)
        } else if num < 32776 {
            // it's a register
            ValueType::Register(num % 32768)
        } else {
            // it's invalid
            panic!("get_value found invalid number at addr {}: {}", addr, num);
        }
    }

    // get the register at the address - fails if not a register
    fn get_register(&self, addr: u16) -> u16 {
        match self.get_ram_value(addr) {
            ValueType::Register(n) => n,
            ValueType::Literal(_) => panic!(),
        }
    }

    // get the value at the address - either grabbing the literal value or
    // traversing into the register itself and using that value
    fn get_value(&self, addr: u16) -> u16 {
        match self.get_ram_value(addr) {
            ValueType::Register(r) => {
                //                info!("(addr={}) register {} read: {}", addr, r, self.registers[r as usize]);
                self.registers[r as usize]
            }
            ValueType::Literal(n) => n,
        }
    }

    fn step(&self, mut addr: u16) -> u16 {
        // grab the instruction to process
        let instruction = self.get_value(addr);

        match instruction {
            0 => {
                // halt
                // stop execution and terminate the program
                log_assembly(addr, "halt");
                addr += 1;
            }
            1 => {
                // set: 1 a b
                // set register <a> to the value of <b>
                let a = self.get_register(addr + 1);
                let b = self.get_ram(addr + 2);

                log_assembly(addr, &format!("set <{}> = {}", a, b));

                addr += 3;
            }
            2 => {
                // push: 2 a
                // push <a> onto the stack
                let a = self.get_ram(addr + 1);

                log_assembly(addr, &format!("push {}", a));

                addr += 2;
            }
            3 => {
                // pop: 3 a
                // remove the top element from the stack and write it into <a>;
                // empty stack = error
                log_assembly(addr, "pop");

                addr += 2;
            }
            4 => {
                // eq: 4 a b c
                // set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
                let a = self.get_ram(addr + 1);
                let b = self.get_ram(addr + 2);
                let c = self.get_ram(addr + 3);

                log_assembly(addr, &format!("eq {}=({} == {})", a, b, c));

                addr += 4;
            }
            5 => {
                // gt: 5 a b c
                // set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
                let a = self.get_ram(addr + 1);
                let b = self.get_ram(addr + 2);
                let c = self.get_ram(addr + 3);

                log_assembly(addr, &format!("gt {}=({} > {})", a, b, c));

                addr += 4;
            }
            6 => {
                // jmp: 6 a
                // jump to <a>
                let a = self.get_ram(addr + 1);
                log_assembly(addr, &format!("jmp {}", a));

                addr += 2;
            }
            7 => {
                // jt: 7 a b
                // if <a> is nonzero, jump to <b>
                let a = self.get_ram(addr + 1);
                let b = self.get_ram(addr + 2);

                log_assembly(addr, &format!("jt ({} != 0 -> {})", a, b));

                addr += 3;
            }
            8 => {
                // jf: 8 a b
                // if <a> is zero, jump to <b>
                let a = self.get_ram(addr + 1);
                let b = self.get_ram(addr + 2);

                log_assembly(addr, &format!("jf ({} == 0 -> {})", a, b));

                addr += 3;
            }
            9 => {
                // add: 9 a b c
                // assign into <a> the sum of <b> and <c> (modulo 32768)
                let a = self.get_register(addr + 1);
                let b = self.get_ram(addr + 2);
                let c = self.get_ram(addr + 3);

                log_assembly(addr, &format!("add <{}> = {} + {}", a, b, c));

                addr += 4;
            }
            10 => {
                // mult: 10 a b c
                // store into <a> the product of <b> and <c> (modulo 32768)
                let a = self.get_register(addr + 1);
                let b = self.get_ram(addr + 2);
                let c = self.get_ram(addr + 3);

                log_assembly(addr, &format!("mult <{}> = {} * {}", a, b, c));

                addr += 4;
            }
            11 => {
                // mod: 11 a b c
                // store into <a> the remainder of <b> divided by <c>
                let a = self.get_register(addr + 1);
                let b = self.get_ram(addr + 2);
                let c = self.get_ram(addr + 3);

                log_assembly(addr, &format!("mod <{}> = {} % {}", a, b, c));

                addr += 4;
            }
            12 => {
                // and: 12 a b c
                // stores into <a> the bitwise and of <b> and <c>
                let a = self.get_register(addr + 1);
                let b = self.get_ram(addr + 2);
                let c = self.get_ram(addr + 3);

                log_assembly(addr, &format!("and <{}> = {} & {}", a, b, c));

                addr += 4;
            }
            13 => {
                // or: 13 a b c
                // stores into <a> the bitwise or of <b> and <c>
                let a = self.get_register(addr + 1);
                let b = self.get_ram(addr + 2);
                let c = self.get_ram(addr + 3);

                log_assembly(addr, &format!("or <{}> = {} | {}", a, b, c));

                addr += 4;
            }
            14 => {
                // not: 14 a b
                // stores 15-bit bitwise inverse of <b> in <a>
                let a = self.get_ram(addr + 1);
                let b = self.get_value(addr + 2);

                log_assembly(addr, &format!("not <{}> = ~{}", a, b));

                addr += 3;
            }
            15 => {
                // rmem: 15 a b
                // read memory at address <b> and write it to <a>
                let b = self.get_ram(addr + 2);

                log_assembly(addr, &format!("rmem {}", b));

                addr += 3;
            }
            16 => {
                // wmem: 16 a b
                // write the value from <b> into memory at address <a>
                let a = self.get_ram(addr + 1);
                let b = self.get_ram(addr + 2);

                // this is how we made a big number
                // (high << 8) + low

                log_assembly(addr, &format!("wmem {} = {}", a, b));

                addr += 3;
            }
            17 => {
                // call: 17 a
                // write the address of the next instruction to the stack and
                // jump to <a>
                let a = self.get_ram(addr + 1);
                log_assembly(addr, &format!("call {}", a));
                addr += 2;
            }
            18 => {
                // ret: 18
                // remove the top element from the stack and jump to it; empty
                // stack = halt
                log_assembly(addr, "ret");
                addr += 1;
            }
            19 => {
                // out: 19 a
                // write the character represented by ascii code <a> to the
                // terminal
                log_assembly(addr, "out");
                addr += 2;
            }
            20 => {
                // in: 20 a
                // read a character from the terminal and write its ascii
                // code to <a>; it can be assumed that once input starts, it
                // will continue until a newline is encountered; this means
                // that you can safely read whole lines from the keyboard
                // instead of having to figure out how to read individual
                // characters
                log_assembly(addr, "in");
                addr += 2;
            }
            21 => {
                // no-op
                // no operation
                log_assembly(addr, "no-op");
                addr += 1;
            }
            n => {
                // uh oh
                eprintln!("unknown instruction: {}", n);
                addr += 1;
            }
        }
        addr
    }
}

fn log_assembly(addr: u16, op: &str) {
    println!("{} {}", addr, op);
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let bin_file = &args[0];
    let mut vm = VM::new(fs::read(bin_file).unwrap());

    let mut addr = 0;
    loop {
        addr = vm.step(addr);
    }
}
