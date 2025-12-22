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
use std::io::{self, Read, Write};

#[derive(Default)]
struct VM {
    ram: Vec<u8>,
    registers: [u16; 8],
    addr: u16, // addr pointer
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

    fn is_halted(&self) -> bool {
        !self.running
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
        println!("running={}, addr={}", self.running, self.addr);
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
            ValueType::Register(r) => self.registers[r as usize],
            ValueType::Literal(n) => n,
        }
    }

    // jump to an ADDRESS
    fn jump(&mut self, addr: u16) {
        debug!("self.jump: jumping to addr {}", addr);
        self.addr = addr;
    }

    fn step(&mut self) {
        assert!(self.running, "tried to step while halted");

        // grab the instruction to process
        let instruction = self.get_value(self.addr);

        match instruction {
            0 => {
                // halt
                // stop execution and terminate the program
                debug!("instruction: {} (addr={})", "halt", self.addr);

                self.running = false;
            }
            1 => {
                // set: 1 a b
                // set register <a> to the value of <b>
                debug!("instruction: {} (addr={})", "set", self.addr);

                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                self.registers[a as usize] = b;

                debug!("set reg {} to {}", a, b);

                self.addr += 3;
            }
            2 => {
                // push: 2 a
                // push <a> onto the stack
                debug!("instruction: {} (addr={})", "push", self.addr);

                let a = self.get_value(self.addr + 1);
                self.push_stack(a);

                self.addr += 2;
            }
            3 => {
                // pop: 3 a
                // remove the top element from the stack and write it into <a>;
                // empty stack = error
                debug!("instruction: {} (addr={})", "pop", self.addr);

                let a = self.get_register(self.addr + 1);
                let elem = self.pop_stack();

                self.registers[a as usize] = elem;

                self.addr += 2;
            }
            4 => {
                // eq: 4 a b c
                // set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
                debug!("instruction: {} (addr={})", "eq", self.addr);

                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                if b == c {
                    self.registers[a as usize] = 1;
                } else {
                    self.registers[a as usize] = 0;
                }

                self.addr += 4;
            }
            5 => {
                // gt: 5 a b c
                // set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
                debug!("instruction: {} (addr={})", "gt", self.addr);

                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                if b > c {
                    self.registers[a as usize] = 1;
                } else {
                    self.registers[a as usize] = 0;
                }

                self.addr += 4;
            }
            6 => {
                // jmp: 6 a
                // jump to <a>
                debug!("instruction: {} (addr={})", "jmp", self.addr);

                let a = self.get_value(self.addr + 1);

                self.jump(a);
            }
            7 => {
                // jt: 7 a b
                // if <a> is nonzero, jump to <b>
                debug!("instruction: {} (addr={})", "jt", self.addr);

                let a = self.get_value(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                trace!("jt: a={}, b={}", a, b);

                if a != 0 {
                    debug!("jt jumped to {}", b);
                    self.jump(b);
                } else {
                    debug!("jt didn't jump");
                    self.addr += 3;
                }
            }
            8 => {
                // jf: 8 a b
                // if <a> is zero, jump to <b>
                debug!("instruction: {} (addr={})", "jf", self.addr);

                let a = self.get_value(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                trace!("jf: a={}, b={}", a, b);

                if a == 0 {
                    debug!("jf jumped to {}", b);
                    self.jump(b);
                } else {
                    debug!("jf didn't jump");
                    self.addr += 3;
                }
            }
            9 => {
                // add: 9 a b c
                // assign into <a> the sum of <b> and <c> (modulo 32768)
                debug!("instruction: {} (addr={})", "add", self.addr);

                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                let sum = (b + c) % 32768;
                self.registers[a as usize] = sum;

                self.addr += 4;
            }
            10 => {
                // mult: 10 a b c
                // store into <a> the product of <b> and <c> (modulo 32768)
                debug!("instruction: {} (addr={})", "mult", self.addr);

                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                let sum = (b as u32 * c as u32) % 32768;
                self.registers[a as usize] = sum as u16;

                self.addr += 4;
            }
            11 => {
                // mod: 11 a b c
                // store into <a> the remainder of <b> divided by <c>
                debug!("instruction: {} (addr={})", "mod", self.addr);

                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                let sum = (b % c) % 32768;
                self.registers[a as usize] = sum;

                self.addr += 4;
            }
            12 => {
                // and: 12 a b c
                // stores into <a> the bitwise and of <b> and <c>
                debug!("instruction: {} (addr={})", "and", self.addr);

                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                let sum = (b & c) % 32768;
                self.registers[a as usize] = sum;

                self.addr += 4;
            }
            13 => {
                // or: 13 a b c
                // stores into <a> the bitwise or of <b> and <c>
                debug!("instruction: {} (addr={})", "or", self.addr);

                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                let sum = (b | c) % 32768;
                self.registers[a as usize] = sum;

                self.addr += 4;
            }
            14 => {
                // not: 14 a b
                // stores 15-bit bitwise inverse of <b> in <a>
                debug!("instruction: {} (addr={})", "not", self.addr);

                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                // XXX possibly wrong?
                let b = !b % 32768;
                self.registers[a as usize] = b;

                self.addr += 3;
            }
            15 => {
                // rmem: 15 a b
                // read memory at address <b> and write it to <a>
                debug!("instruction: {} (addr={})", "rmem", self.addr);

                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                let num = self.get_ram(b);

                self.registers[a as usize] = num;

                self.addr += 3;
            }
            16 => {
                // wmem: 16 a b
                // write the value from <b> into memory at address <a>
                debug!("instruction: {} (addr={})", "wmem", self.addr);

                let a = self.get_value(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                // this is how we made a big number
                // (high << 8) + low

                let high = b >> 8;
                let low = b % 256;

                trace!("setting value {} into ram memory addr {}", b, a);
                trace!("self.ram[({} * 2)] = {}", a, high);
                trace!("self.ram[({} * 2) + 1] = {}", a, low);
                self.ram[(a * 2) as usize] = low as u8;
                self.ram[(a * 2) as usize + 1] = high as u8;

                self.addr += 3;
            }
            17 => {
                // call: 17 a
                // write the address of the next instruction to the stack and
                // jump to <a>
                debug!("instruction: {} (addr={})", "call", self.addr);

                let a = self.get_value(self.addr + 1);
                self.push_stack(self.addr + 2);

                self.jump(a);
            }
            18 => {
                // ret: 18
                // remove the top element from the stack and jump to it; empty
                // stack = halt
                debug!("instruction: {} (addr={})", "ret", self.addr);

                let addr = self.pop_stack();
                self.jump(addr);
            }
            19 => {
                // out: 19 a
                // write the character represented by ascii code <a> to the
                // terminal
                debug!("instruction: {} (addr={})", "out", self.addr);

                let a = self.get_value(self.addr + 1);
                eprint!("{}", a as u8 as char);
                trace!("output: {}", a);

                self.addr += 2;
            }
            20 => {
                // in: 20 a
                // read a character from the terminal and write its ascii
                // code to <a>; it can be assumed that once input starts, it
                // will continue until a newline is encountered; this means
                // that you can safely read whole lines from the keyboard
                // instead of having to figure out how to read individual
                // characters
                debug!("instruction: {} (addr={})", "in", self.addr);

                let a = self.get_register(self.addr + 1);

                // read a single character
                let mut buf: [u8; 1] = [0u8];
                io::stdin()
                    .read_exact(&mut buf)
                    .expect("failed to read 1 char");
                let c = buf[0];

                self.registers[a as usize] = c as u16;

                self.addr += 2;
            }
            21 => {
                // no-op
                // no operation
                debug!("instruction: {} (addr={})", "no-op", self.addr);
                self.addr += 1;
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
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "> {}", record.args()))
        .init();

    let args: Vec<_> = env::args().skip(1).collect();
    let bin_file = &args[0];

    let binary = fs::read(bin_file).unwrap();
    let mut vm = VM::new(binary);

    while !vm.is_halted() {
        vm.step();
    }

    println!("VM finished");
}
