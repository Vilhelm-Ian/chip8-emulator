use crossterm::terminal::SetSize;
use crossterm::{
    event::{poll, read, Event, KeyCode},
    queue,
    style::*,
    terminal::{self, EnterAlternateScreen},
    ExecutableCommand,
};
use rand::prelude::*;
use std::error;
use std::fs;
use std::io::Write;
use std::io::{stdout, Stdout};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let instructions = fs::read("./tetris.rom").unwrap();
    let mut stdout: Stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    stdout.execute(EnterAlternateScreen).unwrap();
    stdout.execute(SetSize(32, 64)).unwrap();
    program(instructions, &mut stdout);
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
    SHRVx(u8),
    SUBN(u8, u8),
    SHL(u8),
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
            return Ok(Instruction::SHRVx(chars_to_hex(&chars[1..=1])? as u8));
        }
        if chars[0] == '8' && chars[3] == '7' {
            return Ok(Instruction::SUBN(
                chars_to_hex(&chars[1..=1])? as u8,
                chars_to_hex(&chars[2..=2])? as u8,
            ));
        }
        if chars[0] == '8' && chars[3] == '8' {
            return Ok(Instruction::SHL(chars_to_hex(&chars[1..1])? as u8));
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

fn program(instructions: Vec<u8>, stdout: &mut Stdout) {
    let mut memory = [0; 4096];
    for (i, instruction) in instructions.iter().enumerate() {
        memory[0x200 + i] = *instruction;
    }
    let mut registers = [0; 17];
    let mut program_counter = 0x200;
    let mut stack_counter = 0;
    let mut i_register = 0;
    let mut stack: [u16; 16] = [0; 16];
    let mut screen = [[0; 64]; 32];
    let delay_timer = Arc::new(Mutex::new(0));
    let sound_timer = Arc::new(Mutex::new(0));
    for (i, font) in FONT.into_iter().enumerate() {
        memory[i] = font;
    }
    {
        let delay_timer = Arc::clone(&delay_timer);
        let sound_timer = Arc::clone(&sound_timer);
        thread::spawn(move || loop {
            let mut delay_timer: u8 = *delay_timer.lock().unwrap();
            delay_timer = delay_timer.saturating_sub(1);
            let mut sound_timer: u8 = *sound_timer.lock().unwrap();
            sound_timer = sound_timer.saturating_sub(1);
            thread::sleep(Duration::from_millis(16));
        });
    }
    let mut current = ' ';
    loop {
        stdout.flush().unwrap();
        if poll(Duration::from_millis(0)).unwrap() {
            if let Event::Key(event) = read().unwrap() {
                if let KeyCode::Char(m) = event.code {
                    stdout.execute(Print(format!("p{}\n\r", current))).unwrap();
                    current = m;
                }
                if event.code == KeyCode::Char('q') {
                    stdout
                        .execute(Print("You pressed 'q'. Exiting...\n"))
                        .unwrap();
                    panic!("done");
                }
            }
        }
        let delay_timer = Arc::clone(&delay_timer);
        let sound_timer = Arc::clone(&sound_timer);
        let hex = numbers_to_hex(
            memory[program_counter as usize],
            memory[program_counter as usize + 1],
        );
        if let Ok(insruction) = Instruction::from_str(&hex) {
            read_instruction(
                insruction,
                &mut program_counter,
                &mut stack_counter,
                &mut registers,
                &mut stack,
                &mut i_register,
                delay_timer,
                sound_timer,
                &mut memory,
                &mut screen,
                stdout,
                &mut current,
            )
            .unwrap();
        };
        program_counter += 2;
    }
}

fn read_instruction(
    instruction: Instruction,
    program_counter: &mut u16,
    stack_counter: &mut u16,
    registers: &mut [u8; 17],
    stack: &mut [u16; 16],
    i_register: &mut u16,
    delay_timer: Arc<Mutex<u8>>,
    sound_timer: Arc<Mutex<u8>>,
    memory: &mut [u8; 4096],
    screen: &mut [[u8; 64]; 32],
    stdout: &mut Stdout,
    current: &mut char,
) -> Result<(), Box<dyn error::Error>> {
    match instruction {
        Instruction::SysAddr(_location) => {
            //
        }
        Instruction::CLS => {
            *screen = [[0; 64]; 32];
        }
        Instruction::RET => {
            *program_counter = stack[*stack_counter as usize];
            *stack_counter -= 1;
        }
        Instruction::JPaddr(location) => {
            *program_counter = location;
            *program_counter -= 2;
        }
        Instruction::CallAddr(location) => {
            *stack_counter += 1;
            stack[*stack_counter as usize] = *program_counter;
            *program_counter = location;
        }
        Instruction::SEVx(register, kk) => {
            if registers[register as usize] == kk {
                *program_counter += 2;
            }
        }
        Instruction::SNEVx(register, kk) => {
            if registers[register as usize] != kk {
                *program_counter += 2;
            }
        }
        Instruction::SEVxVy(register, register2) => {
            if registers[register as usize] == registers[register2 as usize] {
                *program_counter += 2;
            }
        }
        Instruction::LDVx(register, kk) => registers[register as usize] = kk,
        Instruction::ADDVx(register, kk) => {
            registers[register as usize] = registers[register as usize].wrapping_add(kk);
        }
        Instruction::LDVxVy(register, register2) => {
            registers[register as usize] = registers[register2 as usize]
        }
        Instruction::ORVxVy(register, register2) => {
            registers[register as usize] |= registers[register2 as usize]
        }
        Instruction::ANDVxVy(register, register2) => {
            registers[register as usize] &= registers[register2 as usize]
        }
        Instruction::XORVxVy(register, register2) => {
            registers[register as usize] ^= registers[register2 as usize]
        }
        Instruction::ADDVxVy(register, register2) => {
            let carry = registers[register as usize] as u32 + registers[register2 as usize] as u32;
            if carry > 255 {
                registers[0xF] = 1;
            } else {
                registers[0xF] = 0
            }
            registers[register as usize] = registers[register as usize]
                .overflowing_add(registers[register2 as usize])
                .0;
        }
        Instruction::SUBVxVy(register, register2) => {
            if registers[register as usize] >= registers[register2 as usize] {
                registers[0xF] = 1;
            } else {
                registers[0xF] = 0;
            }
            registers[register as usize] = registers[register as usize]
                .overflowing_sub(registers[register2 as usize])
                .0;
        }
        Instruction::SHRVx(register) => {
            let least_significant_beat = registers[register as usize] & 1;
            if least_significant_beat == 1 {
                registers[0xF] = 1;
            } else {
                registers[0xF] = 0;
            }
            registers[register as usize] /= 2;
        }
        Instruction::SUBN(register, register2) => {
            if registers[register2 as usize] >= registers[register as usize] {
                registers[0xF] = 1;
            } else {
                registers[0xF] = 0;
            }
            registers[register as usize] =
                registers[register2 as usize] - registers[register as usize];
        }
        Instruction::SHL(register) => {
            let most_significant_bit = registers[register as usize] >> 7;
            if most_significant_bit == 1 {
                registers[0xF] = 1;
            } else {
                registers[0xF] = 0;
            }
            registers[register as usize] *= 2;
        }
        Instruction::SNE(register, register2) => {
            if registers[register as usize] != registers[register2 as usize] {
                *program_counter += 2;
            }
        }
        Instruction::LDI(nnn) => {
            *i_register = nnn;
        }
        Instruction::JPV0ADDR(nnn) => {
            *program_counter = nnn + registers[0] as u16;
        }
        Instruction::RNDVx(x, kk) => {
            let mut rng = rand::thread_rng();
            let random_number: u8 = rng.gen_range(0..=255);
            registers[x as usize] = random_number & kk;
        }
        Instruction::DRW(x, y, n) => {
            registers[0xF] = 0;
            let y = registers[y as usize] as usize;
            let x = registers[x as usize] as usize;
            let bytes = &memory[*i_register as usize..(*i_register + n as u16) as usize];
            for (i, byte) in bytes.iter().enumerate() {
                for z in 0..8 {
                    let bit = (byte >> (7 - z)) & 1;
                    let new_y = (y + i) % screen.len();
                    let new_x = (x + z) % screen[0].len();
                    let was_on = screen[new_y][new_x] == 1;
                    screen[new_y][new_x] ^= bit;
                    let is_off = screen[new_y][new_x] == 0;
                    if was_on && is_off {
                        registers[0xF] = 1;
                    }
                }
            }
            for line in screen {
                queue!(
                    stdout,
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
        Instruction::SKP(x) => {
            if *current
                == format!("{:X}", registers[x as usize])
                    .chars()
                    .next()
                    .unwrap()
            {
                *current = ' ';
                *program_counter += 2;
            }
        }
        Instruction::SKNP(x) => {
            if *current
                != format!("{:X}", registers[x as usize])
                    .chars()
                    .next()
                    .unwrap()
            {
                *program_counter += 2;
            }
        }
        Instruction::LDVxDT(x) => {
            registers[x as usize] = *delay_timer.lock().unwrap();
        }
        Instruction::LDDTVx(x) => {
            *delay_timer.lock().unwrap() = registers[x as usize];
        }
        Instruction::LDSTVx(x) => {
            *sound_timer.lock().unwrap() = registers[x as usize];
        }
        Instruction::ADDIVx(x) => {
            *i_register += registers[x as usize] as u16;
        }
        Instruction::LDFVx(x) => {
            let value = registers[x as usize];
            *i_register = value as u16 * 5;
        }
        Instruction::LDBVx(x) => {
            let mut x = registers[x as usize];
            let first = x % 10;
            x /= 10;
            let second = x % 10;
            x /= 10;
            let third = x % 10;
            memory[*i_register as usize] = third;
            memory[*i_register as usize + 1] = second;
            memory[*i_register as usize + 2] = first;
        }
        Instruction::LDIVx(x) => {
            for (i, register) in registers.iter().take(x as usize + 1).enumerate() {
                memory[i + (*i_register as usize)] = *register;
            }
        }
        Instruction::LDVxI(x) => {
            for (i, register) in registers.iter_mut().take(x as usize + 1).enumerate() {
                *register = memory[i + (*i_register as usize)];
            }
        }
        Instruction::LDVxK(x) => loop {
            if let Some(val) = handle_input() {
                registers[x as usize] = val;
                break;
            }
        },
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
