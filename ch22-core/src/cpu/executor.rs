use super::interrupt_due_state::*;
use super::registers::*;
use crate::cpu_io::*;
use crate::word::*;

mod accumulator_binary_ops;
mod addressing;
mod memory_util;
mod unary_ops;

use accumulator_binary_ops::*;
use addressing::*;
use memory_util::*;
use unary_ops::*;

use AddressMode::*;
use Instruction::*;
use RegisterType::*;

pub fn execute<IO: CpuIO>(
    io: &mut IO,
    registers: &mut Registers,
    interrupt_due_state: &mut InterruptDueState,
    allow_untested_in_wild: bool,
) {
    let instruction =
        get_next_instruction(io, registers, interrupt_due_state, allow_untested_in_wild);

    instruction.execute(io, registers, interrupt_due_state);

    io.instruction_ended();
}

fn get_next_instruction<IO: CpuIO>(
    io: &mut IO,
    registers: &mut Registers,
    interrupt_due_state: &mut InterruptDueState,
    allow_untested_in_wild: bool,
) -> Instruction {
    if interrupt_due_state.interrupt_due.is_none() {
        let opcode = immediate_fetch(io, &mut registers.program_counter);

        if [0x36, 0x41, 0x56, 0x5e, 0xe1].contains(&opcode) && !allow_untested_in_wild {
            panic!("untested opcode: {:02x}", opcode);
        }

        decode(opcode, registers)
    } else {
        io.phantom_read(registers.program_counter);

        Break(false)
    }
}

fn decode(opcode: u8, registers: &Registers) -> Instruction {
    match opcode {
        // BRK
        0x00 => Break(true),

        // ORA (zp,X)
        0x01 => AccumulatorBinaryOp(or, IndexedIndirect(registers.x)),

        // DOP zp
        0x04 => NopRead(ZeroPage),

        // ORA zp
        0x05 => AccumulatorBinaryOp(or, ZeroPage),

        // ASL zp
        0x06 => ReadModifyWrite(shift_left, ZeroPage),

        // SLO zp
        0x07 => ReadModifyWriteWithAccumulator(shift_left, or, ZeroPage),

        // PHP
        0x08 => PushProcessorFlags,

        // ORA imm
        0x09 => AccumulatorBinaryOp(or, Immediate),

        // ASL A
        0x0a => RegisterUnaryOp(shift_left, Accumulator),

        // ANC imm
        0x0b => AccumulatorBinaryOp(and_negative_carry, Immediate),

        // ORA abs
        0x0d => AccumulatorBinaryOp(or, Absolute),

        // ASL abs
        0x0e => ReadModifyWrite(shift_left, Absolute),

        // BPL rel
        0x10 => Branch(!registers.flags.negative),

        // ORA (zp),Y
        0x11 => AccumulatorBinaryOp(or, IndirectIndexed(registers.y)),

        // ORA zp,X
        0x15 => AccumulatorBinaryOp(or, ZeroPageIndexed(registers.x)),

        // ASL zp,X
        0x16 => ReadModifyWrite(shift_left, ZeroPageIndexed(registers.x)),

        // CLC
        0x18 => SetFlag(set_carry, false),

        // ORA abs,X
        0x1d => AccumulatorBinaryOp(or, AbsoluteIndexed(registers.x)),

        // ASL abs,X
        0x1e => ReadModifyWrite(shift_left, AbsoluteIndexed(registers.x)),

        // ORA abs,Y
        0x19 => AccumulatorBinaryOp(or, AbsoluteIndexed(registers.y)),

        // JSR abs
        0x20 => JumpToSubRoutine,

        // AND (zp,X)
        0x21 => AccumulatorBinaryOp(and, IndexedIndirect(registers.x)),

        // BIT zp
        0x24 => AccumulatorBinaryOp(bit_test, ZeroPage),

        // AND zp
        0x25 => AccumulatorBinaryOp(and, ZeroPage),

        // ROL zp
        0x26 => ReadModifyWrite(rotate_left, ZeroPage),

        // PLP
        0x28 => PullProcessorFlags,

        // AND imm
        0x29 => AccumulatorBinaryOp(and, Immediate),

        // ROL A
        0x2a => RegisterUnaryOp(rotate_left, Accumulator),

        // BIT abs
        0x2c => AccumulatorBinaryOp(bit_test, Absolute),

        // AND abs
        0x2d => AccumulatorBinaryOp(and, Absolute),

        // ROL abs
        0x2e => ReadModifyWrite(rotate_left, Absolute),

        // BMI rel
        0x30 => Branch(registers.flags.negative),

        // AND (zp),Y
        0x31 => AccumulatorBinaryOp(and, IndirectIndexed(registers.y)),

        // AND zp,X
        0x35 => AccumulatorBinaryOp(and, ZeroPageIndexed(registers.x)),

        // ROL zp,X
        0x36 => ReadModifyWrite(rotate_left, ZeroPageIndexed(registers.x)),

        // SEC
        0x38 => SetFlag(set_carry, true),

        // AND abs,Y
        0x39 => AccumulatorBinaryOp(and, AbsoluteIndexed(registers.y)),

        // AND abs,X
        0x3d => AccumulatorBinaryOp(and, AbsoluteIndexed(registers.x)),

        // ROL abs,X
        0x3e => ReadModifyWrite(rotate_left, AbsoluteIndexed(registers.x)),

        // RTI
        0x40 => ReturnFromInterrupt,

        // EOR (zp,X)
        0x41 => AccumulatorBinaryOp(xor, IndexedIndirect(registers.x)),

        // EOR zp
        0x45 => AccumulatorBinaryOp(xor, ZeroPage),

        // LSR zp
        0x46 => ReadModifyWrite(shift_right, ZeroPage),

        // PHA
        0x48 => PushAccumulator,

        // EOR imm
        0x49 => AccumulatorBinaryOp(xor, Immediate),

        // LSR A
        0x4a => RegisterUnaryOp(shift_right, Accumulator),

        // ALR imm
        0x4b => AccumulatorBinaryOp(and_shift_right, Immediate),

        // JMP abs
        0x4c => Jump(Absolute),

        // EOR abs
        0x4d => AccumulatorBinaryOp(xor, Absolute),

        // LSR abs
        0x4e => ReadModifyWrite(shift_right, Absolute),

        // BVC rel
        0x50 => Branch(!registers.flags.overflow),

        // EOR (zp),Y
        0x51 => AccumulatorBinaryOp(xor, IndirectIndexed(registers.y)),

        // EOR zp,X
        0x55 => AccumulatorBinaryOp(xor, ZeroPageIndexed(registers.x)),

        // LSR zp,X
        0x56 => ReadModifyWrite(shift_right, ZeroPageIndexed(registers.x)),

        // CLI
        0x58 => SetFlag(set_interrupt_disable, false),

        // EOR abs,Y
        0x59 => AccumulatorBinaryOp(xor, AbsoluteIndexed(registers.y)),

        // EOR abs,X
        0x5d => AccumulatorBinaryOp(xor, AbsoluteIndexed(registers.x)),

        // LSR abs,X
        0x5e => ReadModifyWrite(shift_right, AbsoluteIndexed(registers.x)),

        // RTS
        0x60 => ReturnFromSubroutine,

        // ADC (zp,X)
        0x61 => AccumulatorBinaryOp(add_with_carry, IndexedIndirect(registers.x)),

        // ADC zp
        0x65 => AccumulatorBinaryOp(add_with_carry, ZeroPage),

        // ROR zp
        0x66 => ReadModifyWrite(rotate_right, ZeroPage),

        // PLA
        0x68 => PullAccumulator,

        // ADC imm
        0x69 => AccumulatorBinaryOp(add_with_carry, Immediate),

        // ROR A
        0x6a => RegisterUnaryOp(rotate_right, Accumulator),

        // JMP (abs)
        0x6c => Jump(Indirect),

        // ADC abs
        0x6d => AccumulatorBinaryOp(add_with_carry, Absolute),

        // ROR abs
        0x6e => ReadModifyWrite(rotate_right, Absolute),

        // BVS rel
        0x70 => Branch(registers.flags.overflow),

        // ADC (zp)
        0x71 => AccumulatorBinaryOp(add_with_carry, IndirectIndexed(registers.y)),

        // ADC zp,X
        0x75 => AccumulatorBinaryOp(add_with_carry, ZeroPageIndexed(registers.x)),

        // ROR zp,X
        0x76 => ReadModifyWrite(rotate_right, ZeroPageIndexed(registers.x)),

        // SEI
        0x78 => SetFlag(set_interrupt_disable, true),

        // ADC abs,Y
        0x79 => AccumulatorBinaryOp(add_with_carry, AbsoluteIndexed(registers.y)),

        // ADC abs,X
        0x7d => AccumulatorBinaryOp(add_with_carry, AbsoluteIndexed(registers.x)),

        // ROR abs,X
        0x7e => ReadModifyWrite(rotate_right, AbsoluteIndexed(registers.x)),

        // STA (zp,X)
        0x81 => Store(registers.accumulator, IndexedIndirect(registers.x)),

        // STY zp
        0x84 => Store(registers.y, ZeroPage),

        // STA zp
        0x85 => Store(registers.accumulator, ZeroPage),

        // STX zp
        0x86 => Store(registers.x, ZeroPage),

        // SAX zp
        0x87 => Store(registers.accumulator & registers.x, ZeroPage),

        // DEY
        0x88 => RegisterUnaryOp(decrement, Y),

        // TXA
        0x8a => TransferRegister(registers.x, Accumulator),

        // STY abs
        0x8c => Store(registers.y, Absolute),

        // STA abs
        0x8d => Store(registers.accumulator, Absolute),

        // STX abs
        0x8e => Store(registers.x, Absolute),

        // BCC rel
        0x90 => Branch(!registers.flags.carry),

        // STA (zp),Y
        0x91 => Store(registers.accumulator, IndirectIndexed(registers.y)),

        // STY zp,X
        0x94 => Store(registers.y, ZeroPageIndexed(registers.x)),

        // STA zp,X
        0x95 => Store(registers.accumulator, ZeroPageIndexed(registers.x)),

        // STX zp,Y
        0x96 => Store(registers.x, ZeroPageIndexed(registers.y)),

        // STA abs,Y
        0x99 => Store(registers.accumulator, AbsoluteIndexed(registers.y)),

        // TYA
        0x98 => TransferRegister(registers.y, Accumulator),

        // TXS
        0x9a => TransferRegisterNoFlags(registers.x, StackPointer),

        // SHY abs,X
        0x9c => StoreHighAddressAndY(AbsoluteIndexed(registers.x)),

        // STA abs,X
        0x9d => Store(registers.accumulator, AbsoluteIndexed(registers.x)),

        // LDY imm
        0xa0 => Load(Y, Immediate),

        // LDA (zp,X)
        0xa1 => Load(Accumulator, IndexedIndirect(registers.x)),

        // LDX imm
        0xa2 => Load(X, Immediate),

        // LDY zp
        0xa4 => Load(Y, ZeroPage),

        // LDA zp
        0xa5 => Load(Accumulator, ZeroPage),

        // LDX zp
        0xa6 => Load(X, ZeroPage),

        // TAY
        0xa8 => TransferRegister(registers.accumulator, Y),

        // LDA imm
        0xa9 => Load(Accumulator, Immediate),

        // TXA
        0xaa => TransferRegister(registers.accumulator, X),

        // LDY abs
        0xac => Load(Y, Absolute),

        // LDA abs
        0xad => Load(Accumulator, Absolute),

        // LDX abs
        0xae => Load(X, Absolute),

        // BCS rel
        0xb0 => Branch(registers.flags.carry),

        // LDA (zp),Y
        0xb1 => Load(Accumulator, IndirectIndexed(registers.y)),

        // LDY zp,X
        0xb4 => Load(Y, ZeroPageIndexed(registers.x)),

        // LDA zp,X
        0xb5 => Load(Accumulator, ZeroPageIndexed(registers.x)),

        // LDX zp,Y
        0xb6 => Load(X, ZeroPageIndexed(registers.y)),

        // CLV
        0xb8 => SetFlag(set_overflow, false),

        // LDA abs,Y
        0xb9 => Load(Accumulator, AbsoluteIndexed(registers.y)),

        // TSX
        0xba => TransferRegister(registers.stack_pointer, X),

        // LDY abs,X
        0xbc => Load(Y, AbsoluteIndexed(registers.x)),

        // LDA abs,X
        0xbd => Load(Accumulator, AbsoluteIndexed(registers.x)),

        // LDX abs,Y
        0xbe => Load(X, AbsoluteIndexed(registers.y)),

        // CPY imm
        0xc0 => Compare(registers.y, Immediate),

        // CMP (zp,X)
        0xc1 => Compare(registers.accumulator, IndexedIndirect(registers.x)),

        // CPY zp
        0xc4 => Compare(registers.y, ZeroPage),

        // CMP zp
        0xc5 => Compare(registers.accumulator, ZeroPage),

        // DEC zp
        0xc6 => ReadModifyWrite(decrement, ZeroPage),

        // INY
        0xc8 => RegisterUnaryOp(increment, Y),

        // CMP abs
        0xc9 => Compare(registers.accumulator, Immediate),

        // DEX
        0xca => RegisterUnaryOp(decrement, X),

        // CPY abs
        0xcc => Compare(registers.y, Absolute),

        // CMP abs
        0xcd => Compare(registers.accumulator, Absolute),

        // DEC abs
        0xce => ReadModifyWrite(decrement, Absolute),

        // BNE rel
        0xd0 => Branch(!registers.flags.zero),

        // CMP (zp),Y
        0xd1 => Compare(registers.accumulator, IndirectIndexed(registers.y)),

        // CMP zp,X
        0xd5 => Compare(registers.accumulator, ZeroPageIndexed(registers.x)),

        // DEC zp,X
        0xd6 => ReadModifyWrite(decrement, ZeroPageIndexed(registers.x)),

        // CLD
        0xd8 => SetFlag(set_decimal_mode, false),

        // CMP abs,Y
        0xd9 => Compare(registers.accumulator, AbsoluteIndexed(registers.y)),

        // NOP abs,X
        0xdc => NopRead(AbsoluteIndexed(registers.x)),

        // CMP abs,X
        0xdd => Compare(registers.accumulator, AbsoluteIndexed(registers.x)),

        // DEC abs,X
        0xde => ReadModifyWrite(decrement, AbsoluteIndexed(registers.x)),

        // CPX imm
        0xe0 => Compare(registers.x, Immediate),

        // SBC (zp,X)
        0xe1 => AccumulatorBinaryOp(subtract_with_carry, IndexedIndirect(registers.x)),

        // CPX zp
        0xe4 => Compare(registers.x, ZeroPage),

        // SBC zp
        0xe5 => AccumulatorBinaryOp(subtract_with_carry, ZeroPage),

        // INC zp
        0xe6 => ReadModifyWrite(increment, ZeroPage),

        // INX
        0xe8 => RegisterUnaryOp(increment, X),

        // SBC imm
        0xe9 => AccumulatorBinaryOp(subtract_with_carry, Immediate),

        // NOP
        0xea => Nop,

        // CPX abs
        0xec => Compare(registers.x, Absolute),

        // SBC abs
        0xed => AccumulatorBinaryOp(subtract_with_carry, Absolute),

        // INC abs
        0xee => ReadModifyWrite(increment, Absolute),

        // BEQ rel
        0xf0 => Branch(registers.flags.zero),

        // SBC (zp),Y
        0xf1 => AccumulatorBinaryOp(subtract_with_carry, IndirectIndexed(registers.y)),

        // SBC zp,X
        0xf5 => AccumulatorBinaryOp(subtract_with_carry, ZeroPageIndexed(registers.x)),

        // INC zp,X
        0xf6 => ReadModifyWrite(increment, ZeroPageIndexed(registers.x)),

        // SED
        0xf8 => SetFlag(set_decimal_mode, true),

        // SBC abs,Y
        0xf9 => AccumulatorBinaryOp(subtract_with_carry, AbsoluteIndexed(registers.y)),

        // SBC abs,X
        0xfd => AccumulatorBinaryOp(subtract_with_carry, AbsoluteIndexed(registers.x)),

        // INC abs,X
        0xfe => ReadModifyWrite(increment, AbsoluteIndexed(registers.x)),

        _ => panic!("Unimplemented opcode: {:#04x}", opcode),
    }
}

impl Instruction {
    pub fn execute<IO: CpuIO>(
        &self,
        io: &mut IO,
        registers: &mut Registers,
        interrupt_due_state: &mut InterruptDueState,
    ) {
        match self {
            NopRead(address_mode) => {
                get_data_with_interrupt_check(
                    io,
                    &mut registers.program_counter,
                    address_mode,
                    registers.flags.interrupt_disable,
                    interrupt_due_state,
                );
            }

            Nop => {
                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                io.phantom_read(registers.program_counter);
            }

            Store(value, address_mode) => {
                let address = get_address(io, &mut registers.program_counter, address_mode);

                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                io.write(address, *value);
            }

            ReadModifyWrite(unary_op, address_mode) => {
                let address = get_address(io, &mut registers.program_counter, address_mode);

                let old_value = io.read(address);

                io.write(address, old_value);

                let new_value = unary_op(&mut registers.flags, old_value);

                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                io.write(address, new_value);
            }

            ReadModifyWriteWithAccumulator(unary_op, accumulator_binary_op, address_mode) => {
                let address = get_address(io, &mut registers.program_counter, address_mode);

                let old_value = io.read(address);

                io.write(address, old_value);

                let new_value = unary_op(&mut registers.flags, old_value);

                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                io.write(address, new_value);

                accumulator_binary_op(&mut registers.flags, &mut registers.accumulator, new_value);
            }

            RegisterUnaryOp(unary_op, register_type) => {
                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                io.phantom_read(registers.program_counter);

                let old_value = registers.get(register_type);

                let new_value = unary_op(&mut registers.flags, old_value);

                registers.set(register_type, new_value);
            }

            AccumulatorBinaryOp(accumulator_binary_op, address_mode) => {
                let operand = get_data_with_interrupt_check(
                    io,
                    &mut registers.program_counter,
                    address_mode,
                    registers.flags.interrupt_disable,
                    interrupt_due_state,
                );

                accumulator_binary_op(&mut registers.flags, &mut registers.accumulator, operand);
            }

            SetFlag(set_flag_fn, value) => {
                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                io.phantom_read(registers.program_counter);

                set_flag_fn(&mut registers.flags, *value);
            }

            Break(from_opcode) => {
                io.phantom_read(registers.program_counter);

                if *from_opcode {
                    registers.program_counter.increment();
                }

                push_word(io, &mut registers.stack_pointer, registers.program_counter);

                let flags = u8::from(registers.flags) | (if *from_opcode { P_BREAK } else { 0 });

                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                push(io, &mut registers.stack_pointer, flags);

                registers.flags.interrupt_disable = true;

                let vector = match interrupt_due_state.interrupt_due {
                    Some(InterruptType::NMI) => NMI_VECTOR,
                    _ => IRQ_BRK_VECTOR,
                };

                interrupt_due_state.interrupt_due = None;

                registers.program_counter = read_word(io, vector);
            }

            JumpToSubRoutine => {
                let new_program_counter_low = immediate_fetch(io, &mut registers.program_counter);

                phantom_stack_read(io, registers.stack_pointer);

                push_word(io, &mut registers.stack_pointer, registers.program_counter);

                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                let new_program_counter_high = immediate_fetch(io, &mut registers.program_counter);

                registers.program_counter = Word(new_program_counter_low, new_program_counter_high);
            }

            Jump(address_mode) => {
                registers.program_counter = get_address_with_interrupt_check(
                    io,
                    &mut registers.program_counter,
                    address_mode,
                    registers.flags.interrupt_disable,
                    interrupt_due_state,
                );
            }

            ReturnFromInterrupt => {
                io.phantom_read(registers.program_counter);

                phantom_stack_read(io, registers.stack_pointer);

                registers.flags = pop(io, &mut registers.stack_pointer).into();

                registers.program_counter = pop_word_with_interrupt_check(
                    io,
                    &mut registers.stack_pointer,
                    registers.flags.interrupt_disable,
                    interrupt_due_state,
                );
            }

            ReturnFromSubroutine => {
                io.phantom_read(registers.program_counter);

                phantom_stack_read(io, registers.stack_pointer);

                registers.program_counter = pop_word(io, &mut registers.stack_pointer);

                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                io.phantom_read(registers.program_counter);

                registers.program_counter.increment();
            }

            PullAccumulator => {
                io.phantom_read(registers.program_counter);

                phantom_stack_read(io, registers.stack_pointer);

                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                registers.accumulator = pop(io, &mut registers.stack_pointer);

                registers.flags.update_zero_negative(registers.accumulator);
            }

            PushAccumulator => {
                io.phantom_read(registers.program_counter);

                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                push(io, &mut registers.stack_pointer, registers.accumulator);
            }

            PullProcessorFlags => {
                io.phantom_read(registers.program_counter);

                phantom_stack_read(io, registers.stack_pointer);

                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                registers.flags = pop(io, &mut registers.stack_pointer).into();
            }

            PushProcessorFlags => {
                io.phantom_read(registers.program_counter);

                let flags = u8::from(registers.flags) | P_BREAK;

                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                push(io, &mut registers.stack_pointer, flags);
            }

            Branch(condition) => {
                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                if !condition {
                    io.phantom_read(registers.program_counter);

                    registers.program_counter.increment();
                } else {
                    registers.program_counter = get_address_with_interrupt_check(
                        io,
                        &mut registers.program_counter,
                        &Relative,
                        registers.flags.interrupt_disable,
                        interrupt_due_state,
                    );
                }
            }

            Compare(register_value, address_mode) => {
                let value = get_data_with_interrupt_check(
                    io,
                    &mut registers.program_counter,
                    address_mode,
                    registers.flags.interrupt_disable,
                    interrupt_due_state,
                );

                registers.flags.carry = *register_value >= value;
                registers.flags.zero = *register_value == value;

                let diff = (register_value).wrapping_sub(value);
                registers.flags.update_negative(diff);
            }

            Load(register_type, address_mode) => {
                let value = get_data_with_interrupt_check(
                    io,
                    &mut registers.program_counter,
                    address_mode,
                    registers.flags.interrupt_disable,
                    interrupt_due_state,
                );

                registers.set(register_type, value);

                registers.flags.update_zero_negative(value);
            }

            TransferRegister(value, register_type) => {
                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                io.phantom_read(registers.program_counter);

                registers.set(register_type, *value);

                registers.flags.update_zero_negative(*value);
            }

            TransferRegisterNoFlags(value, register_type) => {
                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                io.phantom_read(registers.program_counter);

                registers.set(register_type, *value);
            }

            StoreHighAddressAndY(address_mode) => {
                let (address, carried) =
                    address_with_carry(io, &mut registers.program_counter, address_mode);

                update_interrupt_due_state(
                    interrupt_due_state,
                    io,
                    registers.flags.interrupt_disable,
                );

                let Word(low, high) = address;

                if carried {
                    let value = registers.y & high;

                    let address = Word(low, value);

                    io.write(address, value);
                } else {
                    let value = registers.y & high.wrapping_add(1);

                    io.write(address, value);
                };
            }
        }
    }
}

enum Instruction {
    Nop,
    NopRead(AddressMode),
    Store(u8, AddressMode),
    ReadModifyWrite(UnaryOpFn, AddressMode),
    ReadModifyWriteWithAccumulator(UnaryOpFn, AccumulatorBinaryOpFn, AddressMode),
    RegisterUnaryOp(UnaryOpFn, RegisterType),
    AccumulatorBinaryOp(AccumulatorBinaryOpFn, AddressMode),
    SetFlag(SetFlagFn, bool),
    Break(bool),
    JumpToSubRoutine,
    Jump(AddressMode),
    ReturnFromInterrupt,
    ReturnFromSubroutine,
    PullAccumulator,
    PushAccumulator,
    PullProcessorFlags,
    PushProcessorFlags,
    Branch(bool),
    Compare(u8, AddressMode),
    Load(RegisterType, AddressMode),
    TransferRegister(u8, RegisterType),
    TransferRegisterNoFlags(u8, RegisterType),
    StoreHighAddressAndY(AddressMode),
}

pub const NMI_VECTOR: Word = Word(0xfa, 0xff);
pub const RESET_VECTOR: Word = Word(0xfc, 0xff);
pub const IRQ_BRK_VECTOR: Word = Word(0xfe, 0xff);
