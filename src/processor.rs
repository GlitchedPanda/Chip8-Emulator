use std::{fs::File, io::Read};

use rand::random;

use crate::font::FONTSET;

pub struct State<'a> {
    pub vram: &'a [bool; 64*32],
    pub vram_updated: bool
}

enum ProgramCounter {
    Next,
    Skip,
    Nothing
}

pub struct Processor {
    ram: [u8; 4096],
    v: [u8; 16],
    i: usize,
    pc: usize,
    vram: [bool; 64*32],
    vram_updated: bool,
    delay_timer: u8,
    sound_timer: u8,
    stack: [usize; 16],
    sp: usize,
    keys: [bool; 16]
}

impl Processor {
    pub fn new() -> Self {
        
        // Load fontset into ram
        let mut ram = [0u8; 4096];
        for i in 0..FONTSET.len() {
            ram[i] = FONTSET[i];
        }

        Processor {
            ram: ram,
            v: [0u8; 16], // Registers
            i: 0, // Index register
            pc: 0x200, // Program counter
            vram: [false; 64*32],
            vram_updated: false,
            delay_timer: 0u8,
            sound_timer: 0u8,
            stack: [0; 16],
            sp: 0, // Stack pointer
            keys: [false; 16]
        } // Return empty instance of Processor
    }

    fn push(&mut self, value: usize) {
        self.stack[self.sp] = value;
        self.sp += 1;
    }

    fn pop(&mut self) -> usize {
        self.sp -= 1;
        self.stack[self.sp]
    }

    pub fn load(&mut self, path: &String) {
        let mut file = File::open(path).expect("[-] Could not open file");
        let mut buffer = [0u8; 3584];

        let _ = file.read(&mut buffer);

        for i in 0..buffer.len() {
            self.ram[0x200 + i] = buffer[i];
        }
    }
    
    pub fn tick(&mut self) -> State {
        // Emulation cycle
        self.vram_updated = false;

        let opcode = self.get_opcode();
        self.run_opcode(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1; 
        }

        State {
            vram: &self.vram,
            vram_updated: self.vram_updated
        }
    }

    fn get_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16) // The opcode is two bytes
                                                                         // long, so we add them
                                                                         // together. (0xA2 and 0xF0 become 0xA2F0)
    }

    fn run_opcode(&mut self, opcode: u16) { // https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
        //println!("Running opcode: {:04x}", opcode);

        let nibbles = ( // Half a byte is called a nibble
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            (opcode & 0x000F)
        );
        let change_pc = match nibbles {
            (0x0, 0x0, 0x0, 0x0) => { ProgramCounter::Next }, // NOP
            (0x0, 0x0, 0xE, 0x0) => { // CLEAR
                self.vram = [false; 64*32]; 
                self.vram_updated = true;

                ProgramCounter::Next
            },
            (0x0, 0x0, 0xE, 0xE) => { // RET
                let ret_addr: usize = self.pop();
                
                self.pc = ret_addr;
            
                ProgramCounter::Nothing
            },
            (0x1, _, _, _) => { // JMP
                let nnn: u16 = opcode & 0x0FFF; // Gets the address to jump to
                
                self.pc = nnn as usize;

                ProgramCounter::Nothing
            },
            (0x2, _, _ , _) => { // CALL
                let nnn: u16 = opcode & 0x0FFF;
                
                self.push(self.pc);
                self.pc = nnn as usize;

                ProgramCounter::Nothing
            },
            (0x3, _, _, _) => { // Skips the next instruction if Vx == nn
                let x: u16 = nibbles.0;
                let nn: u16 = opcode & 0x00FF;
                
                if self.v[x as usize] == nn as u8 {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            },
            (0x4, _, _, _) => { // Skips the next instruction if Vx != nn
                let x: u16 = nibbles.0;
                let nn: u16 = opcode & 0x00FF;
                
                if self.v[x as usize] != nn as u8 {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            },
            (0x5, _, _, 0x0) => { // Skips the next instruction if Vx == Vy
                let x: u16 = nibbles.1;
                let y: u16 = nibbles.2;
                
                if self.v[x as usize] == self.v[y as usize] {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Nothing
                }
            },
            (0x6, _, _, _) => { // Sets Vx to NN
                let x: u16 = nibbles.1;
                let nn: u16 = opcode & 0x00FF;

                self.v[x as usize] = nn as u8;

                ProgramCounter::Next
            },
            (0x7, _, _, _) => { // Adds NN to Vx
                let x: u16 = nibbles.1;
                let nn: u16 = opcode & 0x00FF;

                self.v[x as usize] += nn as u8;

                ProgramCounter::Next
            },
            (0x8, _, _, 0x0) => { // Sets Vx to the value of Vy
                let x: u16 = nibbles.1;
                let y: u16 = nibbles.2;
                
                self.v[x as usize] = self.v[y as usize];

                ProgramCounter::Next
            },
            (0x8, _, _, 0x1) => { // Sets Vx to Vx OR Vy
                let x: u16 = nibbles.1;
                let y: u16 = nibbles.2;
                
                self.v[x as usize] |= self.v[y as usize];
                ProgramCounter::Next
            },
            (0x8, _, _, 0x2) => { // Sets Vx to Vx AND Vy
                let x: u16 = nibbles.1;
                let y: u16 = nibbles.2;
                
                self.v[x as usize] &= self.v[y as usize];

                ProgramCounter::Next
            },
            (0x8, _, _, 0x3) => { // Sets Vx to Vx XOR Vy
                let x: u16 = nibbles.1;
                let y: u16 = nibbles.2;
                
                self.v[x as usize] ^= self.v[y as usize];

                ProgramCounter::Next
            },
            (0x8, _, _, 0x4) => { // Adds Vy to Vx (can overflow)
                let x: u16 = nibbles.1;
                let y: u16 = nibbles.2;
                
                let (new_vx, carry) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                let new_vf: u8 = if carry { 1 } else { 0 }; // Vf is set to 1 if it overflowed and 0
                                                            // if not.
                self.v[x as usize] = new_vx;
                self.v[0xF] = new_vf;

                ProgramCounter::Next
            },
            (0x8, _, _, 0x5) => { // Subtracts Vy from Vx (can underflow)
                let x: u16 = nibbles.1;
                let y: u16 = nibbles.2;
                
                let (new_vx, borrow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                let new_vf: u8 = if borrow { 0 } else { 1 }; // Vf is set to 0 if there's an
                                                             // underflow and 1 otherwise.
                self.v[x as usize] = new_vx;
                self.v[0xF] = new_vf;

                ProgramCounter::Next
            },
            (0x8, _, _, 0x6) => { // Shifts Vx to the right by one
                let x: u16 = nibbles.1; 
                let lsb = self.v[x as usize] & 0x1; // Least significant bit
                
                self.v[x as usize] >>= 1;
                self.v[0xF] = lsb;

                ProgramCounter::Next
            },
            (0x8, _, _, 0x7) => { // Sets Vx to the value of Vy minus Vx
                let x: u16 = nibbles.1;
                let y: u16 = nibbles.2;

                let (new_vx, borrow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                let new_vf: u8 = if borrow { 0 } else { 1 };

                self.v[x as usize] = new_vx;
                self.v[0xF] = new_vf;

                ProgramCounter::Next
            },
            (0x8, _, _, 0xE) => { // Shifts Vx to the left by one
                let x: u16 = nibbles.1;
                let msb = (self.v[x as usize] >> 7) & 0x1; // Most significant bit
                
                self.v[x as usize] <<= 1; 
                self.v[0xF] = msb;

                ProgramCounter::Next
            },
            (0x9, _, _, 0x0) => { // Skips the next instruction if Vx != Vy 
                let x: u16 = nibbles.1;
                let y: u16 = nibbles.2;

                if self.v[x as usize] != self.v[y as usize] {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            },
            (0xA, _, _, _) => { // Sets I to the address NNN
                let nnn: u16 = opcode & 0x0FFF;
                
                self.i = nnn as usize;

                ProgramCounter::Next
            },
            (0xB, _, _, _) => { // Sets PC to the value of V0 plus NNN
                let nnn: u16 = opcode & 0x0FFF;

                self.pc = (self.v[0x0] as u16 + nnn) as usize;
                
                ProgramCounter::Next
            },
            (0xC, _, _, _) => { // Sets VX to the result of a bitwise and operation on a random number
                let x: u16 = nibbles.1;
                let nn: u16 = opcode & 0x00FF;
                
                let rng: u8 = random();
                self.v[x as usize] = rng & (nn as u8); 

                ProgramCounter::Next
            },
            (0xD, _, _, _) => { // Draws a sprite at coordinate (Vx, Vy) that has a width of 8 pixels and a height of N pixels.
                let x_cord: u16 = nibbles.1;
                let y_cord: u16 = nibbles.2;
                let height: u16 = nibbles.3;

                let mut flipped: bool = false; // We set Vf to to 1 if any screen pixels are flipped from
                                               // set to unset when the sprite is drawn.
                
                for y_line in 0..height {
                    let address: usize = self.i + y_line as usize;
                    let pixels: u8 = self.ram[address];

                    for x_line in 0..8 { 
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_cord + x_line) as usize % 64;
                            let y = (y_cord + y_line) as usize % 32;
                            
                            let index = x + 64 * y;
                            
                            flipped |= self.vram[index];
                            self.vram[index] ^= true;
                            self.vram_updated = true;
                        }
                    }
                }

                if flipped {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }

                ProgramCounter::Next
            },
            (0xE, _, 0x9, 0xE) => { // Skip the next instruction if the key in Vx is pressed
                let x: u16 = nibbles.1;
                let vx: u8 = self.v[x as usize];

                if self.keys[vx as usize] {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            },
            (0xE, _, 0xA, 0x1) => { // Skip the next instruction if the key in Vx is not pressed
                let x: u16 = nibbles.1;
                let vx: u8 = self.v[x as usize];

                if !self.keys[vx as usize] {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            },
            (0xF, _, 0x0, 0x7) => { // Sets Vx to the value of the delay timer.
                let x: u16 = nibbles.1;
                
                self.v[x as usize] = self.delay_timer;

                ProgramCounter::Next
            },
            (0xF, _, 0x0, 0xA) => { // Waits for any key to be pressed, blocking any other
                                    // operation and storing the pressed key in Vx
                let x: u16 = nibbles.1;

                let mut pressed: bool = false;

                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v[x as usize] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if pressed {
                    ProgramCounter::Next
                } else {
                    ProgramCounter::Nothing
                }
            },
            (0xF, _, 0x1, 0x5) => { // Sets the delay timer to Vx
                let x: u16 = nibbles.1;
                self.delay_timer = self.v[x as usize];

                ProgramCounter::Next
            },
            (0xF, _, 0x1, 0x8) => { // Sets the sound timer to Vx
                let x: u16 = nibbles.1;
                self.sound_timer = self.v[x as usize];

                ProgramCounter::Next
            },
            (0xF, _, 0x1, 0xE) => { // Adds Vx to I. Vf isn't affected.
                let x: u16 = nibbles.1;
                let vx: usize = self.v[x as usize] as usize;
                
                self.i = self.i.wrapping_add(vx);

                ProgramCounter::Next
            },
            (0xF, _, 0x2, 0x9) => { // Set I to the font address of character Vx
                let x: u16 = nibbles.1;
                let character: u8 = self.v[x as usize];

                self.i = (character * 5) as usize; // Font sprites take up five bytes each, so
                                                   // their address is just their value times 5.

                ProgramCounter::Next
            },
            (0xF, _, 0x3, 0x3) => { // Stores the binary-coded decimal representation of Vx,
                                    // with the hundreds digit in memory at location in I, 
                                    // the tens digit at location I+1, and the ones digit
                                    // at location I+2
                let x: u16 = nibbles.1;
                let vx: f32 = self.v[x as usize] as f32;
                
                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;
                    
                self.ram[self.i as usize] = hundreds;
                self.ram[(self.i + 1) as usize] = tens;
                self.ram[(self.i + 2) as usize] = ones;

                ProgramCounter::Next
            },
            (0xF, _, 0x5, 0x5) => { // Stores from V0 to VX (including VX) in memory, starting at address I. 
                let x: u16 = nibbles.1;
                let i: usize = self.i;
                for index in 0..=x {
                    self.ram[i + index as usize] = self.v[index as usize];
                }

                ProgramCounter::Next
            },
            (0xF, _, 0x6, 0x5) => { // Fills from V0 to VX (including VX) with values from memory, 
                                    // starting at address I. 
                let x: u16 = nibbles.1;
                let i = self.i;
                for index in 0..=x {
                    self.v[index as usize] = self.ram[i + index as usize];
                }

                ProgramCounter::Next
            },
            (_, _, _, _) => { unimplemented!("[-] Unimplemented opcode: {:04x}", opcode); },
        };

        match change_pc {
            ProgramCounter::Next => self.pc += 2,
            ProgramCounter::Skip => self.pc += 4, // Skips next instruction
            ProgramCounter::Nothing => {},
        }
    }

}
