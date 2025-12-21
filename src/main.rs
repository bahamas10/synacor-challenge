/*!
 * My implementation of the 2012 Synacor VM Challenge.
 *
 * Author: Dave Eddy <ysap@daveeddy.com>
 * Date: December 21, 2025
 * License: MIT
 */

use log::{debug, trace};
use std::env;
use std::fs;

#[derive(Default)]
struct VM {
    ram: Vec<u8>,
    registers: [u16; 8],
    ptr: u16,
    stack: Vec<u16>,
    running: bool,
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

    fn push_stack(&mut self, value: u16) {
        debug!("pushing {} onto the stack", value);
        self.stack.push(value);
    }

    fn pop_stack(&mut self) -> u16 {
        self.stack.pop().expect("stack was empty")
    }

    fn dump_state(&self) {
        println!("loaded rom of size {}", self.ram.len());
        for (i, register) in self.registers.iter().enumerate() {
            println!("register {}: {}", i, register);
        }
        // TODO dump stack
        println!("running={}, ptr={}", self.running, self.ptr);
    }

    fn get_ram(&self, ptr: u16) -> u16 {
        let low = self.ram[ptr as usize] as u16;
        let high = self.ram[ptr as usize + 1] as u16;

        let num = (high << 8) + low;
        trace!(
            "self.get_ram: ptr={} (low={} high={}) num={}",
            ptr, low, high, num
        );

        num
    }

    // get the raw number from the rom
    fn get_ram_value(&self, ptr: u16) -> ValueType {
        let num = self.get_ram(ptr);

        if num < 32768 {
            // it's a literal value
            ValueType::Literal(num)
        } else if num < 32776 {
            // it's a register
            ValueType::Register(num % 32768)
        } else {
            // it's invalid
            panic!("get_value found invalid number at ptr {}: {}", ptr, num);
        }
    }

    // get the register at the pointer - fails if not a register
    fn get_register(&self, ptr: u16) -> u16 {
        match self.get_ram_value(ptr) {
            ValueType::Register(n) => n,
            ValueType::Literal(_) => panic!(),
        }
    }

    // get the value at the pointer - either grabbing the literal value or
    // traversing into the register itself and using that value
    fn get_value(&self, ptr: u16) -> u16 {
        match self.get_ram_value(ptr) {
            ValueType::Register(r) => self.registers[r as usize],
            ValueType::Literal(n) => n,
        }
    }

    // jump to an ADDRESS
    fn jump(&mut self, addr: u16) {
        let ptr = addr * 2;
        debug!("self.jump: jumping to addr {} (ptr={})", addr, ptr);
        self.ptr = ptr;
    }

    fn step(&mut self) {
        assert!(self.running, "tried to step while halted");

        // grab the instruction to process
        let instruction = self.get_value(self.ptr);

        match instruction {
            0 => {
                // halt
                // stop execution and terminate the program
                trace!("ptr={}: halt", self.ptr);

                self.running = false;
            }
            1 => {
                // set: 1 a b
                // set register <a> to the value of <b>
                trace!("ptr={}: set", self.ptr);

                let a = self.get_register(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);

                self.registers[a as usize] = b;

                debug!("set reg {} to {}", a, b);

                self.ptr += 6;
            }
            2 => {
                // push: 2 a
                // push <a> onto the stack
                trace!("ptr={}: push", self.ptr);

                let a = self.get_value(self.ptr + 2);
                self.push_stack(a);

                self.ptr += 4;
            }
            3 => {
                // pop: 3 a
                // remove the top element from the stack and write it into <a>;
                // empty stack = error
                trace!("ptr={}: push", self.ptr);

                let a = self.get_register(self.ptr + 2);
                let elem = self.pop_stack();

                self.registers[a as usize] = elem;

                self.ptr += 4;
            }
            4 => {
                // eq: 4 a b c
                // set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
                trace!("ptr={}: eq", self.ptr);

                let a = self.get_register(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);
                let c = self.get_value(self.ptr + 6);

                if b == c {
                    self.registers[a as usize] = 1;
                } else {
                    self.registers[a as usize] = 0;
                }

                self.ptr += 8;
            }
            5 => {
                // gt: 5 a b c
                // set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
                trace!("ptr={}: gt", self.ptr);

                let a = self.get_register(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);
                let c = self.get_value(self.ptr + 6);

                if b > c {
                    self.registers[a as usize] = 1;
                } else {
                    self.registers[a as usize] = 0;
                }

                self.ptr += 8;
            }
            6 => {
                // jmp: 6 a
                // jump to <a>
                trace!("ptr={}: jmp", self.ptr);

                let a = self.get_value(self.ptr + 2);

                self.jump(a);
            }
            7 => {
                // jt: 7 a b
                // if <a> is nonzero, jump to <b>
                trace!("ptr={}: jt", self.ptr);

                let a = self.get_value(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);

                if a != 0 {
                    debug!("jt jumped to {}", b);
                    self.jump(a);
                } else {
                    self.ptr += 6;
                }
            }
            8 => {
                // jf: 8 a b
                // if <a> is zero, jump to <b>
                trace!("ptr={}: jf", self.ptr);

                let a = self.get_value(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);

                if a == 0 {
                    debug!("jf jumped to {}", b);
                    self.jump(b);
                } else {
                    self.ptr += 6;
                }
            }
            9 => {
                // add: 9 a b c
                // assign into <a> the sum of <b> and <c> (modulo 32768)
                trace!("ptr={}: add", self.ptr);

                let a = self.get_register(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);
                let c = self.get_value(self.ptr + 6);

                let sum = (b + c) % 32768;
                self.registers[a as usize] = sum;

                self.ptr += 6;
            }
            10 => {
                // mult: 10 a b c
                // store into <a> the product of <b> and <c> (modulo 32768)
                trace!("ptr={}: mult", self.ptr);

                let a = self.get_register(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);
                let c = self.get_value(self.ptr + 6);

                let sum = (b * c) % 32768;
                self.registers[a as usize] = sum;

                self.ptr += 6;
            }
            11 => {
                // mod: 11 a b c
                // store into <a> the remainder of <b> divided by <c>
                trace!("ptr={}: mod", self.ptr);

                let a = self.get_register(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);
                let c = self.get_value(self.ptr + 6);

                let sum = (b % c) % 32768;
                self.registers[a as usize] = sum;

                self.ptr += 6;
            }
            12 => {
                // and: 12 a b c
                // stores into <a> the bitwise and of <b> and <c>
                trace!("ptr={}: and", self.ptr);

                let a = self.get_register(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);
                let c = self.get_value(self.ptr + 6);

                let sum = (b & c) % 32768;
                self.registers[a as usize] = sum;

                self.ptr += 6;
            }
            13 => {
                // or: 13 a b c
                // stores into <a> the bitwise or of <b> and <c>
                trace!("ptr={}: or", self.ptr);

                let a = self.get_register(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);
                let c = self.get_value(self.ptr + 6);

                let sum = (b | c) % 32768;
                self.registers[a as usize] = sum;

                self.ptr += 6;
            }
            14 => {
                // not: 14 a b
                // stores 15-bit bitwise inverse of <b> in <a>
                trace!("ptr={}: not", self.ptr);

                let a = self.get_register(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);

                // XXX possibly wrong?
                let b = !b % 32768;
                self.registers[a as usize] = b;

                self.ptr += 4;

                todo!()
            }
            15 => {
                // rmem: 15 a b
                // read memory at address <b> and write it to <a>
                trace!("ptr={}: rmem", self.ptr);

                let a = self.get_register(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);

                let num = self.get_ram(b);

                self.registers[a as usize] = num;

                self.ptr += 6;
            }
            16 => {
                // wmem: 16 a b
                // write the value from <b> into memory at address <a>
                trace!("ptr={}: wmem", self.ptr);

                let a = self.get_value(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);

                // this is how we made a big number
                // (high << 8) + low

                let high = b >> 8;
                let low = b % 256;

                self.ram[a as usize] = high as u8;
                self.ram[(a + 1) as usize] = low as u8;

                self.ptr += 6;
            }
            17 => {
                // call: 17 a
                // write the address of the next instruction to the stack and
                // jump to <a>
                trace!("ptr={}: call", self.ptr);

                let a = self.get_value(self.ptr + 2);
                self.push_stack(self.ptr + 4);

                self.jump(a);
            }
            18 => {
                // ret: 18
                // remove the top element from the stack and jump to it; empty
                // stack = halt
                trace!("ptr={}: ret", self.ptr);

                let ptr = self.pop_stack();
                self.jump(ptr);
            }
            19 => {
                // out: 19 a
                // write the character represented by ascii code <a> to the
                // terminal
                trace!("ptr={}: out", self.ptr);

                let a = self.get_value(self.ptr + 2);
                eprint!("{}", a as u8 as char);
                trace!("output: {}", a);

                self.ptr += 4;
            }
            20 => {
                // in: 20 a
                // read a character from the terminal and write its ascii
                // code to <a>; it can be assumed that once input starts, it
                // will continue until a newline is encountered; this means
                // that you can safely read whole lines from the keyboard
                // instead of having to figure out how to read individual
                // characters
                trace!("ptr={}: in", self.ptr);
                unimplemented!()
            }
            21 => {
                // no-op
                // no operation
                trace!("ptr={}: no op", self.ptr);
                self.ptr += 2;
            }
            n => {
                // uh oh
                self.dump_state();
                panic!("unknown instruction: {}", n);
            }
        }
    }
}

fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().skip(1).collect();
    let bin_file = &args[0];

    let binary = fs::read(bin_file).unwrap();
    let mut vm = VM::new(binary);

    loop {
        vm.step();
    }
}
