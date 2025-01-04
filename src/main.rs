use crossterm::terminal::SetSize;
use crossterm::{
    event::{poll, read, Event, KeyCode},
    queue,
    style::*,
    terminal::{self, EnterAlternateScreen},
    ExecutableCommand,
};
use rand::prelude::*;
use std::env;
use std::error;
use std::fs;
use std::io::Write;
use std::io::{stdout, Stdout};
use std::ops::Index;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct Chip8 {
    program_counter: u16,
    stack_counter: u16,
    registers: [u8; 16],
    stack: [u16; 16],
    i_register: u16,
    delay_timer: Arc<Mutex<u8>>,
    sound_timer: Arc<Mutex<u8>>,
    memory: [u8; 4096],
    screen: [[u8; 64]; 32],
    current: char,
    stdout: Stdout,
}

impl Chip8 {
    fn new() -> Chip8 {
        Self {
            current: ' ',
            registers: [0; 16],
            program_counter: 0x200,
            stack_counter: 0,
            i_register: 0,
            stack: [0; 16],
            screen: [[0; 64]; 32],
            delay_timer: Arc::new(Mutex::new(0)),
            sound_timer: Arc::new(Mutex::new(0)),
            memory: [0; 4096],
            stdout: stdout(),
        }
    }
    fn CLS(&mut self) {
        self.screen = [[0; 64]; 32];
    }
    fn RET(&mut self) {
        if self.stack_counter as usize >= self.stack.len() {
            return ();
        }
        self.program_counter = *self.stack.index(self.stack_counter as usize);
        self.stack_counter = self.stack_counter.overflowing_sub(1).0;
    }
    fn JPaddr(&mut self, location: u16) {
        self.program_counter = location;
        self.program_counter = self.program_counter.overflowing_sub(2).0;
    }
    fn CallAddr(&mut self, location: u16) {
        self.stack_counter = self.stack_counter.overflowing_add(1).0;
        self.stack[self.stack_counter as usize] = self.program_counter;
        self.program_counter = location;
        self.program_counter = self.program_counter.overflowing_sub(2).0;
    }
    fn SEVx(&mut self, register: u8, kk: u8) {
        if self.registers[register as usize] == kk {
            self.program_counter += 2;
        }
    }
    fn SNEVx(&mut self, register: u8, kk: u8) {
        if self.registers[register as usize] != kk {
            self.program_counter += 2;
        }
    }
    fn SEVxVy(&mut self, register: u8, register2: u8) {
        if self.registers[register as usize] == self.registers[register2 as usize] {
            self.program_counter += 2;
        }
    }
    fn LDVx(&mut self, register: u8, kk: u8) {
        self.registers[register as usize] = kk
    }
    fn ADDVx(&mut self, register: u8, kk: u8) {
        self.registers[register as usize] = self.registers[register as usize].wrapping_add(kk);
    }
    fn LDVxVy(&mut self, register: u8, register2: u8) {
        self.registers[register as usize] = self.registers[register2 as usize]
    }
    fn ORVxVy(&mut self, register: u8, register2: u8) {
        self.registers[register as usize] |= self.registers[register2 as usize]
    }
    fn ANDVxVy(&mut self, register: u8, register2: u8) {
        self.registers[register as usize] &= self.registers[register2 as usize]
    }
    fn XORVxVy(&mut self, register: u8, register2: u8) {
        self.registers[register as usize] ^= self.registers[register2 as usize]
    }
    fn ADDVxVy(&mut self, register: u8, register2: u8) {
        let carry =
            self.registers[register as usize] as u32 + self.registers[register2 as usize] as u32;
        if carry > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0
        }
        self.registers[register as usize] = self.registers[register as usize]
            .overflowing_add(self.registers[register2 as usize])
            .0;
    }
    fn SUBVxVy(&mut self, register: u8, register2: u8) {
        if self.registers[register as usize] >= self.registers[register2 as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[register as usize] = self.registers[register as usize]
            .overflowing_sub(self.registers[register2 as usize])
            .0;
    }
    fn SHRVx(&mut self, register: u8, register_2: u8) {
        // todo!() MAKE THIS CONFIGURABL FOR THE USER
        self.registers[register as usize] = self.registers[register_2 as usize];
        let least_significant_beat = self.registers[register as usize] & 1;
        self.registers[register as usize] /= 2;
        if least_significant_beat == 1 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }
    fn SUBN(&mut self, register: u8, register2: u8) {
        if self.registers[register2 as usize] > self.registers[register as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[register as usize] = self.registers[register2 as usize]
            .overflowing_sub(self.registers[register as usize])
            .0;
    }
    fn SHL(&mut self, register: u8, register_2: u8) {
        // todo!() MAKE THIS CONFIGURABL FOR THE USER
        self.registers[register as usize] = self.registers[register_2 as usize];
        let most_significant_bit = self.registers[register as usize] >> 7;
        self.registers[register as usize] *= 2;
        if most_significant_bit == 1 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }
    fn SNE(&mut self, register: u8, register2: u8) {
        if self.registers[register as usize] != self.registers[register2 as usize] {
            self.program_counter += 2;
        }
    }
    fn LDI(&mut self, nnn: u16) {
        self.i_register = nnn;
    }
    fn JPV0ADDR(&mut self, nnn: u16) {
        self.program_counter = nnn + self.registers[0] as u16;
    }
    fn RNDVx(&mut self, x: u8, kk: u8) {
        let mut rng = rand::thread_rng();
        let random_number: u8 = rng.gen_range(0..=255);
        self.registers[x as usize] = random_number & kk;
    }
    fn DRW(&mut self, x: u8, y: u8, n: u8) {
        self.registers[0xF] = 0;
        let y = self.registers[y as usize] as usize;
        let x = self.registers[x as usize] as usize;
        let bytes = &self.memory[self.i_register as usize..(self.i_register + n as u16) as usize];
        for (i, byte) in bytes.iter().enumerate() {
            for z in 0..8 {
                let bit = (byte >> (7 - z)) & 1;
                let new_y = (y + i) % self.screen.len();
                let new_x = (x + z) % self.screen[0].len();
                let was_on = self.screen[new_y][new_x] == 1;
                self.screen[new_y][new_x] ^= bit;
                let is_off = self.screen[new_y][new_x] == 0;
                if was_on && is_off {
                    self.registers[0xF] = 1;
                }
            }
        }
        for line in self.screen {
            queue!(
                self.stdout,
                Print(format!(
                    "{}\n\r",
                    line.iter()
                        .map(|l| {
                            if *l == 0 {
                                ' '
                            } else {
                                '#'
                            }
                        })
                        .collect::<String>()
                ))
            )
            .unwrap();
        }
    }
    fn SKP(&mut self, x: u8) {
        if self.current
            == format!("{:X}", self.registers[x as usize])
                .chars()
                .next()
                .unwrap()
        {
            self.current = ' ';
            self.program_counter += 2;
        }
    }
    fn SKNP(&mut self, x: u8) {
        if self.current
            != format!("{:X}", self.registers[x as usize])
                .chars()
                .next()
                .unwrap()
        {
            self.program_counter += 2;
        }
    }
    fn LDVxDT(&mut self, x: u8) {
        self.registers[x as usize] = *self.delay_timer.lock().unwrap();
    }
    fn LDDTVx(&mut self, x: u8) {
        *self.delay_timer.lock().unwrap() = self.registers[x as usize];
    }
    fn LDSTVx(&mut self, x: u8) {
        *self.sound_timer.lock().unwrap() = self.registers[x as usize];
    }
    fn ADDIVx(&mut self, x: u8) {
        self.i_register += self.registers[x as usize] as u16;
    }
    fn LDFVx(&mut self, x: u8) {
        let value = self.registers[x as usize];
        self.i_register = value as u16 * 5;
    }
    fn LDBVx(&mut self, x: u8) {
        let mut x = self.registers[x as usize];
        let first = x % 10;
        x /= 10;
        let second = x % 10;
        x /= 10;
        let third = x % 10;
        self.memory[self.i_register as usize] = third;
        self.memory[self.i_register as usize + 1] = second;
        self.memory[self.i_register as usize + 2] = first;
    }
    fn LDIVx(&mut self, x: u8) {
        for (i, register) in self.registers.iter().take(x as usize + 1).enumerate() {
            self.memory[i + (self.i_register as usize)] = *register;
        }
    }
    fn LDVxI(&mut self, x: u8) {
        for (i, register) in self.registers.iter_mut().take(x as usize + 1).enumerate() {
            *register = self.memory[i + (self.i_register as usize)];
        }
    }
    fn LDVxK(&mut self, x: u8) {
        loop {
            if let Some(val) = handle_input() {
                self.registers[x as usize] = val;
                break;
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    // Get the file path from the arguments
    let file_path = &args[1];
    let instructions = fs::read(file_path).unwrap();
    let mut stdout: Stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    stdout.execute(EnterAlternateScreen).unwrap();
    stdout.execute(SetSize(32, 64)).unwrap();
    program(instructions);
}

pub const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

enum Instruction {
    SysAddr(u16),
    RET,
    CLS,
    JPaddr(u16),
    CallAddr(u16),
    SEVx(u8, u8),
    SNEVx(u8, u8),
    SEVxVy(u8, u8),
    LDVx(u8, u8),
    ADDVx(u8, u8),
    LDVxVy(u8, u8),
    ORVxVy(u8, u8),
    ANDVxVy(u8, u8),
    XORVxVy(u8, u8),
    ADDVxVy(u8, u8),
    SUBVxVy(u8, u8),
    SHRVx(u8, u8),
    SUBN(u8, u8),
    SHL(u8, u8),
    SNE(u8, u8),
    LDI(u16),
    JPV0ADDR(u16),
    RNDVx(u8, u8),
    DRW(u8, u8, u8),
    SKP(u8),
    SKNP(u8),
    LDVxDT(u8),
    LDVxK(u8),
    LDDTVx(u8),
    LDSTVx(u8),
    ADDIVx(u8),
    LDFVx(u8),
    LDBVx(u8),
    LDIVx(u8),
    LDVxI(u8),
}

struct ParseInstructionError;

impl FromStr for Instruction {
    type Err = ParseInstructionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars: [char; 4] = {
            let mut chars = s.chars();
            [
                chars.next().ok_or(ParseInstructionError)?,
                chars.next().ok_or(ParseInstructionError)?,
                chars.next().ok_or(ParseInstructionError)?,
                chars.next().ok_or(ParseInstructionError)?,
            ]
        };
        if chars == ['0', '0', 'E', '0'] {
            return Ok(Instruction::CLS);
        };
        if chars == ['0', '0', 'E', 'E'] {
            return Ok(Instruction::RET);
        };
        if chars[0] == '0' {
            return Ok(Instruction::SysAddr(chars_to_hex(&chars[1..])?));
        }
        if chars[0] == '1' {
            return Ok(Instruction::JPaddr(chars_to_hex(&chars[1..])?));
        }
        if chars[0] == '2' {
            return Ok(Instruction::CallAddr(chars_to_hex(&chars[1..])?));
        }
        if chars[0] == '3' {
            return Ok(Instruction::SEVx(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..])? as u8,
            ));
        }
        if chars[0] == '4' {
            return Ok(Instruction::SNEVx(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..])? as u8,
            ));
        }
        if chars[0] == '5' && chars[3] == '5' {
            return Ok(Instruction::SEVxVy(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
            ));
        }
        if chars[0] == '6' {
            return Ok(Instruction::LDVx(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..])? as u8,
            ));
        }
        if chars[0] == '7' {
            return Ok(Instruction::ADDVx(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..])? as u8,
            ));
        }
        if chars[0] == '8' && chars[3] == '0' {
            return Ok(Instruction::LDVxVy(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
            ));
        }
        if chars[0] == '8' && chars[3] == '1' {
            return Ok(Instruction::ORVxVy(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
            ));
        }
        if chars[0] == '8' && chars[3] == '2' {
            return Ok(Instruction::ANDVxVy(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
            ));
        }
        if chars[0] == '8' && chars[3] == '3' {
            return Ok(Instruction::XORVxVy(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
            ));
        }
        if chars[0] == '8' && chars[3] == '4' {
            return Ok(Instruction::ADDVxVy(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
            ));
        }
        if chars[0] == '8' && chars[3] == '5' {
            return Ok(Instruction::SUBVxVy(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
            ));
        }
        if chars[0] == '8' && chars[3] == '6' {
            return Ok(Instruction::SHRVx(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[1..=1])? as u8,
            ));
        }
        if chars[0] == '8' && chars[3] == '7' {
            return Ok(Instruction::SUBN(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
            ));
        }
        if chars[0] == '8' && chars[3] == '8' {
            return Ok(Instruction::SHL(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
            ));
        }
        if chars[0] == '9' && chars[3] == '0' {
            return Ok(Instruction::SNE(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
            ));
        }
        if chars[0] == 'A' {
            return Ok(Instruction::LDI(chars_to_hex(&chars[1..])?));
        }
        if chars[0] == 'B' {
            return Ok(Instruction::JPV0ADDR(chars_to_hex(&chars[1..])?));
        }
        if chars[0] == 'C' {
            return Ok(Instruction::RNDVx(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..])? as u8,
            ));
        }
        if chars[0] == 'D' {
            return Ok(Instruction::DRW(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
                chars_to_hex(&chars[3..=3])? as u8,
            ));
        }
        if chars[0] == 'E' && chars[2] == '9' && chars[3] == 'E' {
            return Ok(Instruction::SKP(chars_to_hex(&chars[1..=1])? as u8));
        }
        if chars[0] == 'E' && chars[2] == 'A' && chars[3] == '1' {
            return Ok(Instruction::SKNP(chars_to_hex(&chars[1..=1])? as u8));
        }
        if chars[0] == 'F' && chars[2] == '0' && chars[3] == '7' {
            return Ok(Instruction::LDVxDT(chars_to_hex(&chars[1..=1])? as u8));
        }
        if chars[0] == 'F' && chars[2] == '0' && chars[3] == 'A' {
            return Ok(Instruction::LDVxK(chars_to_hex(&chars[1..=1])? as u8));
        }
        if chars[0] == 'F' && chars[2] == '1' && chars[3] == '5' {
            return Ok(Instruction::LDDTVx(chars_to_hex(&chars[1..=1])? as u8));
        }
        if chars[0] == 'F' && chars[2] == '1' && chars[3] == '8' {
            return Ok(Instruction::LDSTVx(chars_to_hex(&chars[1..=1])? as u8));
        }
        if chars[0] == 'F' && chars[2] == '1' && chars[3] == 'E' {
            return Ok(Instruction::ADDIVx(chars_to_hex(&chars[1..=1])? as u8));
        }
        if chars[0] == 'F' && chars[2] == '2' && chars[3] == '9' {
            return Ok(Instruction::LDFVx(chars_to_hex(&chars[1..=1])? as u8));
        }
        if chars[0] == 'F' && chars[2] == '3' && chars[3] == '3' {
            return Ok(Instruction::LDBVx(chars_to_hex(&chars[1..=1])? as u8));
        }
        if chars[0] == 'F' && chars[2] == '5' && chars[3] == '5' {
            return Ok(Instruction::LDIVx(chars_to_hex(&chars[1..=1])? as u8));
        }
        if chars[0] == 'F' && chars[2] == '6' && chars[3] == '5' {
            return Ok(Instruction::LDVxI(chars_to_hex(&chars[1..=1])? as u8));
        }
        Err(ParseInstructionError)
    }
}

fn chars_to_hex(chars: &[char]) -> Result<u16, ParseInstructionError> {
    u16::from_str_radix(&chars.iter().collect::<String>(), 16).map_err(|_| ParseInstructionError)
}

fn numbers_to_hex(num_1: u8, num_2: u8) -> String {
    let num_1 = format!("{:X}", num_1);
    let num_1 = if num_1.len() == 1 {
        format!("0{}", num_1)
    } else {
        num_1
    };
    let num_2 = format!("{:X}", num_2);
    let num_2 = if num_2.len() == 1 {
        format!("0{}", num_2)
    } else {
        num_2
    };
    format!("{}{}", num_1, num_2)
}

fn program(instructions: Vec<u8>) {
    let mut chip8 = Chip8::new();
    for (i, instruction) in instructions.iter().enumerate() {
        chip8.memory[0x200 + i] = *instruction;
    }
    for (i, font) in FONT.into_iter().enumerate() {
        chip8.memory[i] = font;
    }
    {
        let delay_timer = Arc::clone(&chip8.delay_timer);
        let sound_timer = Arc::clone(&chip8.sound_timer);
        thread::spawn(move || loop {
            let mut delay_timer: u8 = *delay_timer.lock().unwrap();
            delay_timer = delay_timer.saturating_sub(1);
            let mut sound_timer: u8 = *sound_timer.lock().unwrap();
            sound_timer = sound_timer.saturating_sub(1);
            thread::sleep(Duration::from_millis(16));
        });
    }
    loop {
        chip8.stdout.flush().unwrap();
        if poll(Duration::from_millis(0)).unwrap() {
            if let Event::Key(event) = read().unwrap() {
                if let KeyCode::Char(m) = event.code {
                    chip8
                        .stdout
                        .execute(Print(format!("p{}\n\r", chip8.current)))
                        .unwrap();
                    chip8.current = m;
                }
                if event.code == KeyCode::Char('q') {
                    chip8
                        .stdout
                        .execute(Print("You pressed 'q'. Exiting...\n"))
                        .unwrap();
                    panic!("done");
                }
            }
        }
        let hex = numbers_to_hex(
            chip8.memory[chip8.program_counter as usize],
            chip8.memory[chip8.program_counter as usize + 1],
        );
        if let Ok(insruction) = Instruction::from_str(&hex) {
            read_instruction(insruction, &mut chip8).unwrap();
        };
        chip8.program_counter = chip8.program_counter.overflowing_add(2).0;
    }
}

fn read_instruction(
    instruction: Instruction,
    chip8: &mut Chip8,
) -> Result<(), Box<dyn error::Error>> {
    match instruction {
        Instruction::SysAddr(_location) => {
            //
        }
        Instruction::CLS => chip8.CLS(),
        Instruction::RET => chip8.RET(),
        Instruction::JPaddr(location) => chip8.JPaddr(location),
        Instruction::CallAddr(location) => chip8.CallAddr(location),
        Instruction::SEVx(register, kk) => chip8.SEVx(register, kk),
        Instruction::SNEVx(register, kk) => chip8.SNEVx(register, kk),
        Instruction::SEVxVy(register, register2) => chip8.SEVxVy(register, register2),
        Instruction::LDVx(register, kk) => chip8.registers[register as usize] = kk,
        Instruction::ADDVx(register, kk) => chip8.ADDVx(register, kk),
        Instruction::LDVxVy(register, register2) => chip8.LDVxVy(register, register2),
        Instruction::ORVxVy(register, register2) => chip8.ORVxVy(register, register2),
        Instruction::ANDVxVy(register, register2) => chip8.ANDVxVy(register, register2),
        Instruction::XORVxVy(register, register2) => chip8.XORVxVy(register, register2),
        Instruction::ADDVxVy(register, register2) => chip8.ADDVxVy(register, register2),
        Instruction::SUBVxVy(register, register2) => chip8.SUBVxVy(register, register2),
        Instruction::SHRVx(register, register_2) => chip8.SHRVx(register, register_2),
        Instruction::SUBN(register, register2) => chip8.SUBN(register, register2),
        Instruction::SHL(register, register_2) => chip8.SHL(register, register_2),
        Instruction::SNE(register, register2) => chip8.SNE(register, register2),
        Instruction::LDI(nnn) => chip8.LDI(nnn),
        Instruction::JPV0ADDR(nnn) => chip8.JPV0ADDR(nnn),
        Instruction::RNDVx(x, kk) => chip8.RNDVx(x, kk),
        Instruction::DRW(x, y, n) => chip8.DRW(x, y, n),
        Instruction::SKP(x) => chip8.SKP(x),
        Instruction::SKNP(x) => chip8.SKNP(x),
        Instruction::LDVxDT(x) => chip8.LDVxDT(x),
        Instruction::LDDTVx(x) => chip8.LDDTVx(x),
        Instruction::LDSTVx(x) => chip8.LDSTVx(x),
        Instruction::ADDIVx(x) => chip8.ADDIVx(x),
        Instruction::LDFVx(x) => chip8.LDFVx(x),
        Instruction::LDBVx(x) => chip8.LDBVx(x),
        Instruction::LDIVx(x) => chip8.LDIVx(x),
        Instruction::LDVxI(x) => chip8.LDVxI(x),
        Instruction::LDVxK(x) => chip8.LDVxK(x),
    };
    Ok(())
}

fn handle_input() -> Option<u8> {
    if let Event::Key(event) = read().unwrap() {
        match event.code {
            KeyCode::Char('1') => Some(0x1),
            KeyCode::Char('2') => Some(0x2),
            KeyCode::Char('3') => Some(0x3),
            KeyCode::Char('4') => Some(0xC),
            KeyCode::Char('Q') => Some(0x4),
            KeyCode::Char('W') => Some(0x5),
            KeyCode::Char('E') => Some(0x6),
            KeyCode::Char('R') => Some(0xD),
            KeyCode::Char('A') => Some(0x7),
            KeyCode::Char('S') => Some(0x8),
            KeyCode::Char('D') => Some(0x9),
            KeyCode::Char('F') => Some(0xE),
            KeyCode::Char('Z') => Some(0xA),
            KeyCode::Char('X') => Some(0x0),
            KeyCode::Char('C') => Some(0xB),
            KeyCode::Char('V') => Some(0xF),
            _ => None,
        }
    } else {
        None
    }
}
