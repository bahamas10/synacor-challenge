/*!
 * My implementation of the 2012 Synacor VM Challenge.
 *
 * Author: Dave Eddy <ysap@daveeddy.com>
 * Date: December 21, 2025
 * License: MIT
 */

use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Read, Write};

#[derive(Default, Serialize, Deserialize)]
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

    fn is_halted(&self) -> bool {
        !self.running
    }

    fn push_stack(&mut self, value: u16) {
        trace!("pushing {} onto the stack", value);
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
        for (i, value) in self.stack.iter().enumerate() {
            println!("stack {}: {}", i, value);
        }
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
            ValueType::Register(r) => {
                info!(
                    "(addr={}) register {} read: {}",
                    addr, r, self.registers[r as usize]
                );
                self.registers[r as usize]
            }
            ValueType::Literal(n) => n,
        }
    }

    // set a register to a value
    fn set_register(&mut self, register: u16, value: u16) {
        //info!("register {} write: {}", register, value);
        self.registers[register as usize] = value;
    }

    // jump to an ADDRESS
    fn jump(&mut self, addr: u16) {
        trace!("self.jump: jumping to addr {}", addr);
        self.addr = addr;
    }

    fn log_assembly(&self, op: &str) {
        let w = self.level;
        debug!("{} {:<w$} {}", " ", self.addr, op);
    }

    fn step(&mut self) {
        assert!(self.running, "tried to step while halted");

        // grab the instruction to process
        let instruction = self.get_value(self.addr);

        match instruction {
            0 => {
                // halt
                // stop execution and terminate the program
                self.log_assembly("halt");

                self.running = false;
            }
            1 => {
                // set: 1 a b
                // set register <a> to the value of <b>
                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                self.log_assembly(&format!("set <{}> = {}", a, b));

                self.set_register(a, b);

                self.addr += 3;
            }
            2 => {
                // push: 2 a
                // push <a> onto the stack
                let a = self.get_value(self.addr + 1);
                self.log_assembly(&format!("push {}", a));

                self.push_stack(a);

                self.addr += 2;
            }
            3 => {
                // pop: 3 a
                // remove the top element from the stack and write it into <a>;
                // empty stack = error
                let a = self.get_register(self.addr + 1);
                let elem = self.pop_stack();

                self.log_assembly(&format!(
                    "pop writing {} into <{}>",
                    elem, a
                ));

                self.set_register(a, elem);

                self.addr += 2;
            }
            4 => {
                // eq: 4 a b c
                // set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                self.log_assembly(&format!("eq ({} == {})", b, c));

                if b == c {
                    self.set_register(a, 1);
                } else {
                    self.set_register(a, 0);
                }

                self.addr += 4;
            }
            5 => {
                // gt: 5 a b c
                // set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                self.log_assembly(&format!("gt ({} > {})", b, c));

                if b > c {
                    self.set_register(a, 1);
                } else {
                    self.set_register(a, 0);
                }

                self.addr += 4;
            }
            6 => {
                // jmp: 6 a
                // jump to <a>
                let a = self.get_value(self.addr + 1);
                self.log_assembly(&format!("jmp <{}>", a));

                self.jump(a);
            }
            7 => {
                // jt: 7 a b
                // if <a> is nonzero, jump to <b>
                let a = self.get_value(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                trace!("jt: a={}, b={}", a, b);
                self.log_assembly(&format!("jt ({} != 0 -> {})", a, b));

                if a != 0 {
                    trace!("jt jumped to {}", b);
                    self.jump(b);
                } else {
                    trace!("jt didn't jump");
                    self.addr += 3;
                }
            }
            8 => {
                // jf: 8 a b
                // if <a> is zero, jump to <b>
                let a = self.get_value(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                trace!("jf: a={}, b={}", a, b);
                self.log_assembly(&format!("jf ({} == 0 -> {})", a, b));

                if a == 0 {
                    trace!("jf jumped to {}", b);
                    self.jump(b);
                } else {
                    trace!("jf didn't jump");
                    self.addr += 3;
                }
            }
            9 => {
                // add: 9 a b c
                // assign into <a> the sum of <b> and <c> (modulo 32768)
                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                self.log_assembly(&format!("add <{}> = {} + {}", a, b, c));

                let sum = (b + c) % 32768;
                self.set_register(a, sum);

                self.addr += 4;
            }
            10 => {
                // mult: 10 a b c
                // store into <a> the product of <b> and <c> (modulo 32768)
                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                self.log_assembly(&format!("mult <{}> = {} * {}", a, b, c));

                let sum = (b as u32 * c as u32) % 32768;
                self.set_register(a, sum as u16);

                self.addr += 4;
            }
            11 => {
                // mod: 11 a b c
                // store into <a> the remainder of <b> divided by <c>
                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                self.log_assembly(&format!("mod <{}> = {} % {}", a, b, c));

                let sum = (b % c) % 32768;
                self.set_register(a, sum);

                self.addr += 4;
            }
            12 => {
                // and: 12 a b c
                // stores into <a> the bitwise and of <b> and <c>
                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                self.log_assembly(&format!("and <{}> = {} & {}", a, b, c));

                let sum = (b & c) % 32768;
                self.set_register(a, sum);

                self.addr += 4;
            }
            13 => {
                // or: 13 a b c
                // stores into <a> the bitwise or of <b> and <c>
                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);
                let c = self.get_value(self.addr + 3);

                self.log_assembly(&format!("or <{}> = {} | {}", a, b, c));

                let sum = (b | c) % 32768;
                self.set_register(a, sum);

                self.addr += 4;
            }
            14 => {
                // not: 14 a b
                // stores 15-bit bitwise inverse of <b> in <a>
                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                self.log_assembly(&format!("not <{}> = ~{}", a, b));

                let b = !b % 32768;
                self.set_register(a, b);

                self.addr += 3;
            }
            15 => {
                // rmem: 15 a b
                // read memory at address <b> and write it to <a>
                let a = self.get_register(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                let num = self.get_ram(b);

                self.log_assembly(&format!("rmem <{}> = {}", a, num));

                self.set_register(a, num);

                self.addr += 3;
            }
            16 => {
                // wmem: 16 a b
                // write the value from <b> into memory at address <a>
                let a = self.get_value(self.addr + 1);
                let b = self.get_value(self.addr + 2);

                // this is how we made a big number
                // (high << 8) + low

                let high = b >> 8;
                let low = b % 256;

                trace!("setting value {} into ram memory addr {}", b, a);
                trace!("self.ram[({} * 2)] = {}", a, high);
                trace!("self.ram[({} * 2) + 1] = {}", a, low);

                self.log_assembly(&format!("wmem {} = {}", a, b));

                self.ram[(a * 2) as usize] = low as u8;
                self.ram[(a * 2) as usize + 1] = high as u8;

                self.addr += 3;
            }
            17 => {
                // call: 17 a
                // write the address of the next instruction to the stack and
                // jump to <a>

                let a = self.get_value(self.addr + 1);

                self.log_assembly(&format!("call {}", a));

                if a == 6049 {
                    // LOL - game genie
                    /*
                    self.registers[0] = 6;
                    self.addr += 2;
                    return;
                    */
                }

                self.push_stack(self.addr + 2);

                self.level += 1;
                self.jump(a);
            }
            18 => {
                // ret: 18
                // remove the top element from the stack and jump to it; empty
                // stack = halt
                let addr = self.pop_stack();
                self.log_assembly(&format!("ret ({})", addr));
                self.level -= 1;
                self.jump(addr);
            }
            19 => {
                // out: 19 a
                // write the character represented by ascii code <a> to the
                // terminal
                self.log_assembly("out");

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
                self.log_assembly("in");

                let a = self.get_register(self.addr + 1);

                // read a single character - try from input buffer and fallback
                // to stdin
                let (c, color) = if !self.input_buffer.is_empty() {
                    // input buffer
                    (self.input_buffer.remove(0), 31)
                } else {
                    // stdin
                    let mut buf: [u8; 1] = [0u8];
                    io::stdin()
                        .read_exact(&mut buf)
                        .expect("failed to read 1 char");

                    // allow user to send commands to the VM itself
                    if buf[0] == b'/' {
                        let mut cmd = String::new();
                        io::stdin().read_line(&mut cmd).unwrap();
                        let cmd = cmd.trim();

                        self.process_internal_command(cmd);
                        return;
                    }

                    (buf[0], 32)
                };

                eprint!("\x1b[{}m{}\x1b[0m", color, c as char);

                self.set_register(a, c as u16);

                self.addr += 2;
            }
            21 => {
                // no-op
                // no operation
                self.log_assembly("no-op");
                self.addr += 1;
            }
            n => {
                // uh oh
                self.dump_state();
                panic!("unknown instruction: {}", n);
            }
        }
    }

    fn process_internal_command(&mut self, s: &str) {
        trace!("internal command: {}", s);

        let cmd: Vec<_> = s.split_whitespace().collect();

        match cmd[0] {
            "dump" => self.dump_state(),
            "set" => {
                // set the register
                let register: u16 = cmd[1].parse().unwrap();
                let value: u16 = cmd[2].parse().unwrap();
                println!("updating register {}: {}", register, value);
                self.set_register(register, value);
            }
            "save" => {
                let file = cmd[1];
                if fs::exists(file).unwrap() {
                    println!("file already exists, doing nothing");
                    return;
                }
                fs::write(file, &self.ram).unwrap();
                println!("file saved to {}", file);
            }
            "export" => {
                let file = cmd[1];
                if fs::exists(file).unwrap() {
                    println!("file already exists, doing nothing");
                    return;
                }
                let data = serde_json::to_string(&self).unwrap();
                fs::write(file, &data).unwrap();
                println!("file saved to {}", file);
            }
            cmd => panic!("unknown internal command: {}", cmd),
        }
    }
}

fn main() {
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "> {}", record.args()))
        .init();

    let args: Vec<_> = env::args().skip(1).collect();

    let file = &args[0];

    let mut vm = if file.ends_with(".json") {
        let data = fs::read_to_string(file).unwrap();
        serde_json::from_str(&data).unwrap()
    } else {
        let binary = fs::read(file).unwrap();
        VM::new(binary)
    };

    // command file given as arg2
    if let Some(f) = args.get(1) {
        let input_buffer = fs::read(f).unwrap();
        vm.input_buffer = input_buffer;
    }

    while !vm.is_halted() {
        vm.step();
    }

    println!("VM finished");
}
