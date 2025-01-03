use rand::prelude::*;
use std::error;

fn main() {
    println!("Hello, world!");
}

enum Instruction {
    RET,
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

fn program() {
    let mut memory = [0; 4096];
    let mut registers = [0; 16];
    let program_counter = 0;
    let mut delay_timer = 0;
    let mut sound_timer = 0;
    let stack_counter = 0;
    let mut i_register = 0;
    let mut stack: [u16; 16] = [0; 16];
}

fn read_instruction(
    instruction: Instruction,
    program_counter: &mut u16,
    stack_counter: &mut u16,
    registers: &mut [u8; 16],
    stack: &mut [u16; 16],
    i_register: &mut u16,
    delay_timer: &mut u8,
    sound_timer: &mut u8,
    memory: &mut [u8; 4096],
) -> Result<(), Box<dyn error::Error>> {
    match instruction {
        Instruction::RET => {
            *program_counter = stack[*stack_counter as usize];
            *stack_counter -= 1;
        }
        Instruction::JPaddr(location) => *program_counter = location,
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
        Instruction::ADDVx(register, kk) => registers[register as usize] += kk,
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
                registers[0xF] = 1
            } else {
                registers[0xF] = 0
            }
            registers[register as usize] += registers[register2 as usize]
        }
        Instruction::SUBVxVy(register, register2) => {
            registers[register as usize] -= registers[register2 as usize];
            if registers[register as usize] > registers[register2 as usize] {
                registers[0xF] = 1;
            } else {
                registers[0xF] = 0;
            }
        }
        Instruction::SHRVx(register) => {
            let least_significant_beat = registers[register as usize] & 1;
            if least_significant_beat == 0 {
                registers[0xF] = 1;
            } else {
                registers[0xF] = 0;
            }
            registers[register as usize] /= 2;
        }
        Instruction::SUBN(register, register2) => {
            registers[register as usize] =
                registers[register2 as usize] - registers[register as usize];
            if registers[register2 as usize] > registers[register as usize] {
                registers[0xF] = 1;
            } else {
                registers[0xF] = 0;
            }
        }
        Instruction::SHL(register) => {
            let most_significant_bit = (registers[register as usize] & 0b10000000) >> 7;
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
            todo!()
        }
        Instruction::SKP(x) => {
            todo!()
        }
        Instruction::SKNP(x) => {
            todo!()
        }
        Instruction::LDVxDT(x) => {
            registers[x as usize] = *delay_timer;
        }
        Instruction::LDVxK(x) => {
            todo!()
        }
        Instruction::LDDTVx(x) => {
            *delay_timer = registers[x as usize];
        }
        Instruction::LDSTVx(x) => {
            *sound_timer = registers[x as usize];
        }
        Instruction::ADDIVx(x) => {
            *i_register += registers[x as usize] as u16;
        }
        Instruction::LDFVx(x) => {
            todo!()
        }
        Instruction::LDBVx(x) => {
            todo!()
        }
        Instruction::LDIVx(x) => {
            for (i, register) in registers.iter().take(x as usize).enumerate() {
                memory[i + (*i_register as usize)] = *register;
            }
            todo!()
        }
        Instruction::LDVxI(x) => {
            todo!()
        }
    };
    Ok(())
}
