/**
 * My implementation of the 2012 Synacor VM Challenge.
 *
 * Author: Dave Eddy <ysap@daveeddy.com>
 * Date: December 21, 2025
 * License: MIT
 */

use std::env;
use std::fs;
use log::{debug, trace};

struct VM {
    rom: Vec<u8>,
    registers: [u16; 8],
    ptr: u16,
    running: bool,
}

#[allow(dead_code)]
impl VM {
    fn new(rom: Vec<u8>) -> Self {
        Self { rom, registers: [0; 8], ptr: 0, running: true }
    }

    fn dump_state(&self) {
        println!("loaded rom of size {}", self.rom.len());
        for (i, register) in self.registers.iter().enumerate() {
            println!("register {}: {}", i, register);
        }
        println!("running={}, ptr={}", self.running, self.ptr);
    }

    fn get_value(&self, ptr: u16) -> u16 {
        let low = self.rom[ptr as usize] as u16;
        let high = self.rom[ptr as usize + 1] as u16;

        trace!("get_value = low={}, high={}", low, high);

        (high << 8) + low
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
                let a = self.get_value(self.ptr + 2);
                let b = self.get_value(self.ptr + 4);
                self.registers[(a % 32768) as usize] = b;

                debug!("set reg {} to {}", a, b);

                self.ptr += 6;
            }
            2 => {
                // push: 2 a
                // push <a> onto the stack
                unimplemented!()
            }
            3 => {
                // pop: 3 a
                // remove the top element from the stack and write it into <a>;
                // empty stack = error
                unimplemented!()
            }
            4 => {
                // eq: 4 a b c
                // set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
                unimplemented!()
            }
            5 => {
                // gt: 5 a b c
                // set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
                unimplemented!()
            }
            6 => {
                // jmp: 6 a
                // jump to <a>
                unimplemented!()
            }
            7 => {
                // jt: 7 a b
                // if <a> is nonzero, jump to <b>
                unimplemented!()
            }
            8 => {
                // jf: 8 a b
                // if <a> is zero, jump to <b>
                unimplemented!()
            }
            9 => {
                // add: 9 a b c
                // assign into <a> the sum of <b> and <c> (modulo 32768)
                unimplemented!()
            }
            10 => {
                // mult: 10 a b c
                // store into <a> the product of <b> and <c> (modulo 32768)
                unimplemented!()
            }
            11 => {
                // mod: 11 a b c
                // store into <a> the remainder of <b> divided by <c>
                unimplemented!()
            }
            12 => {
                // and: 12 a b c
                // stores into <a> the bitwise and of <b> and <c>
                unimplemented!()
            }
            13 => {
                // or: 13 a b c
                // stores into <a> the bitwise or of <b> and <c>
                unimplemented!()
            }
            14 => {
                // not: 14 a b
                // stores 15-bit bitwise inverse of <b> in <a>
                unimplemented!()
            }
            15 => {
                // rmem: 15 a b
                // read memory at address <b> and write it to <a>
                unimplemented!()
            }
            16 => {
                // wmem: 16 a b
                // write the value from <b> into memory at address <a>
                unimplemented!()
            }
            17 => {
                // call: 17 a
                // write the address of the next instruction to the stack and
                // jump to <a>
                trace!("ptr={}: call", self.ptr);
                unimplemented!()
            }
            18 => {
                // ret: 18
                // remove the top element from the stack and jump to it; empty
                // stack = halt
                trace!("ptr={}: ret", self.ptr);
                unimplemented!()
            }
            19 => {
                // out: 19 a
                // write the character represented by ascii code <a> to the
                // terminal
                trace!("ptr={}: out", self.ptr);

                let a = self.get_value(self.ptr + 2);
                print!("{}", a as u8 as char);

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

//    vm.dump_state();
    loop {
        vm.step();
//        vm.dump_state();
    }
}
