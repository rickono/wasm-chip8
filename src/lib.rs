mod utils;

use wasm_bindgen::prelude::*;
use rand;
use rand::Rng;
use std::fs;
use std::env;
extern crate console_error_panic_hook;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Computer {
    registers: [u16; 16],
    pc: usize,
    register_i: usize,
    delay_timer: u8,
    sound_timer: u8,
    memory: [u8; 4096],
    keys: [bool; 16],
    pixels: [u64; 32],
    stack: Vec<u16>,
    sprite_locations: [usize; 16],
}

#[wasm_bindgen]
impl Computer {
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        let mut memory = [0; 4096];
        let mut sprite_locations = [0; 16];
        const SPRITES: [[u8; 5]; 16] =[
            [0xF0, 0x90, 0x90, 0x90, 0xF0],
            [0x20, 0x60, 0x20, 0x20, 0x70],
            [0xF0, 0x10, 0xF0, 0x80, 0xF0],
            [0xF0, 0x10, 0xF0, 0x10, 0xF0],
            [0x90, 0x90, 0xF0, 0x10, 0x10],
            [0xF0, 0x80, 0xF0, 0x10, 0xF0],
            [0xF0, 0x80, 0xF0, 0x90, 0xF0],
            [0xF0, 0x10, 0x20, 0x40, 0x40],
            [0xF0, 0x90, 0xF0, 0x90, 0xF0],
            [0xF0, 0x90, 0xF0, 0x10, 0xF0],
            [0xF0, 0x90, 0xF0, 0x90, 0x90],
            [0xE0, 0x90, 0xE0, 0x90, 0xE0],
            [0xF0, 0x80, 0x80, 0x80, 0xF0],
            [0xE0, 0x90, 0x90, 0x90, 0xE0],
            [0xF0, 0x80, 0xF0, 0x80, 0xF0],
            [0xF0, 0x80, 0xF0, 0x80, 0x80]
        ];
        for (number, sprite) in SPRITES.iter().enumerate() {
            memory[number*5..number*5+5].copy_from_slice(sprite);
            sprite_locations[number] = number * 5;
        }
        Computer {
            registers: [0; 16],
            register_i: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            memory,
            keys: [false; 16],
            pixels: [0; 32],
            stack: Vec::new(),
            sprite_locations,
        }
    }

    pub fn registers(&self) -> *const u16 {
        self.registers.as_ptr()
    }

    pub fn keypress(&mut self, key: usize) {
        self.keys[key] = true;
    }

    pub fn load(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            let addr = 0x200 + i;
            println!("Loading {} into {}", byte, addr);
            if addr < 4096 {
                self.memory[addr] = byte;
            } else {
                break;
            }
        }
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn tick(&mut self) -> u16 {
        let instruction = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
        // println!("{:X?}", instruction);
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        self.run_instruction(instruction);
        instruction
    }

    pub fn pixels(&self) -> *const u64 {
        self.pixels.as_ptr()
    }

    pub fn memory(&self) -> *const u8 {
        self.memory.as_ptr()
    }

    pub fn i(&self) -> usize {
        self.register_i
    }

    fn run_instruction(&mut self, instruction: u16) {
        let hex = (
            ((instruction & 0xF000) >> 12) as u8,
            ((instruction & 0x0F00) >> 8) as u8,
            ((instruction & 0x00F0) >> 4) as u8,
            (instruction & 0x000F) as u8
        );
        let x = hex.1;
        let y = hex.2;
        let nnn = instruction & 0xFFF;
        let nn = instruction & 0xFF;
        let n = instruction & 0xF;
        println!("{:?}", hex);
        match hex {
            (0, 0, 0xE, 0) => self.ex_00e0(),
            (0, 0, 0xE, 0xE) => self.ex_00ee(),
            (1, ..) => self.ex_1nnn(nnn),
            (2, ..) => self.ex_2nnn(nnn),
            (3, ..) => self.ex_3xnn(x, nn),
            (4, ..) => self.ex_4xnn(x, nn),
            (5, _, _, 0) => self.ex_5xy0(x, y),
            (6, ..) => self.ex_6xnn(x, nn),
            (7, ..) => self.ex_7xnn(x, nn),
            (8, _, _, 0) => self.ex_8xy0(x, y),
            (8, _, _, 1) => self.ex_8xy1(x, y),
            (8, _, _, 2) => self.ex_8xy2(x, y),
            (8, _, _, 3) => self.ex_8xy3(x, y),
            (8, _, _, 4) => self.ex_8xy4(x, y),
            (8, _, _, 5) => self.ex_8xy5(x, y),
            (8, _, _, 6) => self.ex_8xy6(x, y),
            (8, _, _, 7) => self.ex_8xy7(x, y),
            (8, _, _, 0xE) => self.ex_8xye(x, y),
            (9, _, _, 0) => self.ex_9xy0(x, y),
            (0xA, ..) => self.ex_annn(nnn),
            (0xB, ..) => self.ex_bnnn(nnn),
            (0xC, ..) => self.ex_cxnn(x, nn),
            (0xD, ..) => self.ex_dxyn(x, y, n),
            (0xE, _, 9, _) => self.ex_ex9e(x),
            (0xE, _, 0xA, 1) => self.ex_exa1(x),
            (0xF, _, 0, 7) => self.ex_fx07(x),
            (0xF, _, 0, 0xA) => self.ex_fx0a(x),
            (0xF, _, 1, 5) => self.ex_fx15(x),
            (0xF, _, 1, 8) => self.ex_fx18(x),
            (0xF, _, 1, 0xE) => self.ex_fx1e(x),
            (0xF, _, 2, 9) => self.ex_fx29(x),
            (0xF, _, 3, 3) => self.ex_fx33(x),
            (0xF, _, 5, 5) => self.ex_fx55(x),
            (0xF, _, 6, 5) => self.ex_fx65(x),
            _ => ()
        }
    }

    fn reg(&self, register: u8) -> u16 {
        self.registers[register as usize]
    }

    fn set(&mut self, register: u8, value: u16) {
        self.registers[register as usize] = value;
    }

    fn step(&mut self) {
        self.pc += 0x02;
    }

    fn ex_00e0(&mut self) {
        self.pixels = [0; 32];
        self.step();
    }

    fn ex_00ee(&mut self) {
        match self.stack.pop() {
            Some(address) => self.pc = address as usize,
            None => panic!(),
        }
    }
    
    fn ex_1nnn(&mut self, nnn: u16) {
        self.pc = nnn as usize;
    }

    fn ex_2nnn(&mut self, nnn: u16) {
        self.stack.push((self.pc + 0x02) as u16);
        self.pc = nnn as usize;
    }


    fn ex_3xnn(&mut self, x: u8, nn: u16) {
        if self.reg(x) == nn {
            self.step();
        }
        self.step();
    }

    fn ex_4xnn(&mut self, x: u8, nn: u16) {
        if self.reg(x) != nn {
            self.step();
        }
        self.step();
    }

    fn ex_5xy0(&mut self, x: u8, y: u8) {
        if self.reg(x) == self.reg(y) {
            self.step();
        }
        self.step();
    }

    fn ex_6xnn(&mut self, x: u8, nn: u16) {
        self.set(x, nn);
        self.step();
    }

    fn ex_7xnn(&mut self, x: u8, nn: u16) {
        self.set(x, (self.reg(x) + nn) & 0xFF);
        self.step();
    }

    fn ex_8xy0(&mut self, x: u8, y: u8) {
        self.set(x, self.reg(y));
        self.step();
    }

    fn ex_8xy1(&mut self, x: u8, y: u8) {
        self.set(x, self.reg(x) | self.reg(y));
        self.step();
    }

    fn ex_8xy2(&mut self, x: u8, y: u8) {
        self.set(x, self.reg(x) & self.reg(y));
        self.step();
    }

    fn ex_8xy3(&mut self, x: u8, y: u8) {
        self.set(x, self.reg(x) ^ self.reg(y));
        self.step();
    }

    fn ex_8xy4(&mut self, x: u8, y: u8) {
        let sum = self.reg(x) + self.reg(y);
        self.set(x, sum & 0xFF);
        let carry = (sum & 0xFF00 > 0) as u16;
        self.set(0xF, carry);
        self.step();
    }

    // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    fn ex_8xy5(&mut self, x: u8, y: u8) {
        let vx = self.reg(x);
        let vy = self.reg(y);
        if vx > vy {
            self.set(0xF, 1);
        } else {
            self.set(0xF, 0);
            let diff = (self.reg(x) - self.reg(y)) & 0xFF;
            self.set(x, diff);
        }
        self.step();
    }

    fn ex_8xy6(&mut self, x: u8, y: u8) {
        self.set(0xF, self.reg(x) & 0b1);
        self.set(x, self.reg(x) >> 1);
        self.step();
    }

    fn ex_8xy7(&mut self, x: u8, y: u8) {
        let vx = self.reg(x);
        let vy = self.reg(y);
        if vy > vx {
            self.set(0xF, 1);
        } else {
            self.set(0xF, 0);
            let diff =  (self.reg(y) - self.reg(x)) & 0xFF;
            self.set(x, diff);
        }
        self.step();
    }

    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    fn ex_8xye(&mut self, x: u8, y: u8) {
        self.set(0xF, (self.reg(x) & 0x0080) >> 7);
        self.set(x, (self.reg(x) << 1) & 0xFF);
        self.step();
    }

    fn ex_9xy0(&mut self, x: u8, y: u8) {
        if self.reg(x) != self.reg(y) {
            self.step();
        }
        self.step();
    }

    fn ex_annn(&mut self, nnn: u16) {
        self.register_i = nnn as usize;
        self.step();
    }

    fn ex_bnnn(&mut self, nnn: u16) {
        self.pc = (nnn + self.reg(0)) as usize;
        self.step();
    }

    fn ex_cxnn(&mut self, x: u8, nn: u16) {
        let mut rng = rand::thread_rng();
        let rand: u8 = rng.gen();
        self.set(x, nn & rand as u16);
        self.step();
    }

    fn ex_dxyn(&mut self, x: u8, y: u8, n: u16) {
        let loc_x = self.reg(x);
        let loc_y = self.reg(y);
        let sprite = &self.memory[self.register_i..self.register_i+n as usize];
        for (row, &byte) in sprite.iter().enumerate() {
            if row + loc_y as usize >= 32 {
                break;
            }
            // x = 0 -> all the way to left, shift left by 64 - 8
            // x = 56 -> all the way to right, shift left by 0
            // The new row is the XOR of the sprite with current
            let shift = 64 - 8 - loc_x as i16;
            let new_row;
            if shift > 0 {
                new_row = self.pixels[row + loc_y as usize] ^ ((byte as u64) << shift);
            } else {
                new_row = self.pixels[row + loc_y as usize] ^ ((byte as u64) >> -shift as i16);
            }
            self.pixels[row + loc_y as usize] = new_row;
            if new_row ^ self.pixels[row + loc_y as usize] & self.pixels[row + loc_y as usize] > 0 {
                self.registers[0xF] = 1;
            }
        }
        self.step();
    }

    fn ex_ex9e(&mut self, x: u8) {
        if self.keys[self.reg(x) as usize] {
            self.step();
        }
        self.step();
    }

    fn ex_exa1(&mut self, x: u8) {
        if !self.keys[self.reg(x) as usize] {
            self.step();
        }
        self.step();
    }

    fn ex_fx07(&mut self, x: u8) {
        self.set(x, self.delay_timer.into());
        self.step();
    }

    fn ex_fx0a(&mut self, x: u8) {
        match self.keys.iter().position(|&key| key == true) {
            Some(key) => self.set(x, key as u16),
            None => (),
        }
        self.step();
    }

    fn ex_fx15(&mut self, x: u8) {
        self.delay_timer = self.reg(x) as u8;
        self.step();
    }

    fn ex_fx18(&mut self, x: u8) {
        self.sound_timer = self.reg(x) as u8;
        self.step();
    }

    fn ex_fx1e(&mut self, x: u8) {
        self.register_i += self.reg(x) as usize;
        self.step();
    }

    fn ex_fx29(&mut self, x: u8) {
        self.register_i = self.sprite_locations[x as usize];
        self.step();
    }

    fn ex_fx33(&mut self, x: u8) {
        let vx = self.reg(x);
        self.memory[self.register_i] = (vx / 100) as u8;
        self.memory[self.register_i + 1] = ((vx % 100) / 10) as u8;
        self.memory[self.register_i + 2] = (vx % 10) as u8;
        self.step();
    }

    fn ex_fx55(&mut self, x: u8) {
        for offset in 0..x+1 {
            self.memory[self.register_i + offset as usize] = self.reg(offset as u8) as u8;
        }
        self.step();
    }

    fn ex_fx65(&mut self, x: u8) {
        for offset in 0..x+1 {
            self.set(offset, self.memory[self.register_i + offset as usize].into());
        }
        self.step();
    }
}