use crate::bus::*;

use super::registers::*;

mod addressing;
mod binary_ops;
mod immediate_access;
mod stack_access;
mod unary_ops;

use addressing::*;
use binary_ops::*;
use immediate_access::*;
use stack_access::*;
use unary_ops::*;

use AddressMode::*;
use Instruction::*;

pub fn interrupt<T: BusTrait>(bus: &mut T, registers: &mut Registers, nmi: bool) {
    bus.phantom_read(registers.program_counter);

    Break(false, 0, if nmi { NMI_VECTOR } else { IRQ_BRK_VECTOR }).execute(bus, registers);

    bus.complete();
}

pub fn execute<T: BusTrait>(bus: &mut T, registers: &mut Registers, allow_untested_in_wild: bool) {
    let opcode = read_immediate(bus, &mut registers.program_counter);

    if [0x35, 0x36, 0x41, 0x56, 0x5e, 0xe1].contains(&opcode) && !allow_untested_in_wild {
        panic!("untested opcode: {:02x}", opcode);
    }

    let instruction = decode(opcode, registers);

    instruction.execute(bus, registers);

    bus.complete();
}

fn decode(opcode: u8, registers: &Registers) -> Instruction {
    match opcode {
        // BRK
        0x00 => Break(true, P_BREAK, IRQ_BRK_VECTOR),

        // ORA (zp,X)
        0x01 => AccumulatorBinaryOp(or, IndexedIndirect(registers.x_index)),

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
        0x0a => AccumulatorUnaryOp(shift_left),

        // ANC imm
        0x0b => AccumulatorBinaryOp(and_negative_carry, Immediate),

        // ORA abs
        0x0d => AccumulatorBinaryOp(or, Absolute),

        // ASL abs
        0x0e => ReadModifyWrite(shift_left, Absolute),

        // BPL rel
        0x10 => Branch(!registers.processor_flags.negative),

        // ORA (zp),Y
        0x11 => AccumulatorBinaryOp(or, IndirectIndexed(registers.y_index)),

        // ORA zp,X
        0x15 => AccumulatorBinaryOp(or, ZeroPageIndexed(registers.x_index)),

        // ASL zp,X
        0x16 => ReadModifyWrite(shift_left, ZeroPageIndexed(registers.x_index)),

        // CLC
        0x18 => SetFlag(set_carry, false),

        // ORA abs,X
        0x1d => AccumulatorBinaryOp(or, AbsoluteIndexed(registers.x_index)),

        // ASL abs,X
        0x1e => ReadModifyWrite(shift_left, AbsoluteIndexed(registers.x_index)),

        // ORA abs,Y
        0x19 => AccumulatorBinaryOp(or, AbsoluteIndexed(registers.y_index)),

        // JSR abs
        0x20 => JumpToSubRoutine,

        // AND (zp,X)
        0x21 => AccumulatorBinaryOp(and, IndexedIndirect(registers.x_index)),

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
        0x2a => AccumulatorUnaryOp(rotate_left),

        // BIT abs
        0x2c => AccumulatorBinaryOp(bit_test, Absolute),

        // AND abs
        0x2d => AccumulatorBinaryOp(and, Absolute),

        // ROL abs
        0x2e => ReadModifyWrite(rotate_left, Absolute),

        // BMI rel
        0x30 => Branch(registers.processor_flags.negative),

        // AND (zp),Y
        0x31 => AccumulatorBinaryOp(and, IndirectIndexed(registers.y_index)),

        // AND zp,X
        0x35 => AccumulatorBinaryOp(and, ZeroPageIndexed(registers.x_index)),

        // ROL zp,X
        0x36 => ReadModifyWrite(rotate_left, ZeroPageIndexed(registers.x_index)),

        // SEC
        0x38 => SetFlag(set_carry, true),

        // AND abs,Y
        0x39 => AccumulatorBinaryOp(and, AbsoluteIndexed(registers.y_index)),

        // AND abs,X
        0x3d => AccumulatorBinaryOp(and, AbsoluteIndexed(registers.x_index)),

        // ROL abs,X
        0x3e => ReadModifyWrite(rotate_left, AbsoluteIndexed(registers.x_index)),

        // RTI
        0x40 => ReturnFromInterrupt,

        // EOR (zp,X)
        0x41 => AccumulatorBinaryOp(xor, IndexedIndirect(registers.x_index)),

        // EOR zp
        0x45 => AccumulatorBinaryOp(xor, ZeroPage),

        // LSR zp
        0x46 => ReadModifyWrite(shift_right, ZeroPage),

        // PHA
        0x48 => PushAccumulator,

        // EOR imm
        0x49 => AccumulatorBinaryOp(xor, Immediate),

        // LSR A
        0x4a => AccumulatorUnaryOp(shift_right),

        // ALR imm
        0x4b => AccumulatorBinaryOp(and_shift_right, Immediate),

        // JMP abs
        0x4c => Jump(Absolute),

        // EOR abs
        0x4d => AccumulatorBinaryOp(xor, Absolute),

        // LSR abs
        0x4e => ReadModifyWrite(shift_right, Absolute),

        // BVC rel
        0x50 => Branch(!registers.processor_flags.overflow),

        // EOR (zp),Y
        0x51 => AccumulatorBinaryOp(xor, IndirectIndexed(registers.y_index)),

        // EOR zp,X
        0x55 => AccumulatorBinaryOp(xor, ZeroPageIndexed(registers.x_index)),

        // LSR zp,X
        0x56 => ReadModifyWrite(shift_right, ZeroPageIndexed(registers.x_index)),

        // CLI
        0x58 => SetFlag(set_interrupt_disable, false),

        // EOR abs,Y
        0x59 => AccumulatorBinaryOp(xor, AbsoluteIndexed(registers.y_index)),

        // EOR abs,X
        0x5d => AccumulatorBinaryOp(xor, AbsoluteIndexed(registers.x_index)),

        // LSR abs,X
        0x5e => ReadModifyWrite(shift_right, AbsoluteIndexed(registers.x_index)),

        // RTS
        0x60 => ReturnFromSubroutine,

        // ADC (zp,X)
        0x61 => AccumulatorBinaryOp(add_with_carry, IndexedIndirect(registers.x_index)),

        // ADC zp
        0x65 => AccumulatorBinaryOp(add_with_carry, ZeroPage),

        // ROR zp
        0x66 => ReadModifyWrite(rotate_right, ZeroPage),

        // PLA
        0x68 => PullAccumulator,

        // ADC imm
        0x69 => AccumulatorBinaryOp(add_with_carry, Immediate),

        // ROR A
        0x6a => AccumulatorUnaryOp(rotate_right),

        // JMP (abs)
        0x6c => Jump(Indirect),

        // ADC abs
        0x6d => AccumulatorBinaryOp(add_with_carry, Absolute),

        // ROR abs
        0x6e => ReadModifyWrite(rotate_right, Absolute),

        // BVS rel
        0x70 => Branch(registers.processor_flags.overflow),

        // ADC (zp)
        0x71 => AccumulatorBinaryOp(add_with_carry, IndirectIndexed(registers.y_index)),

        // ADC zp,X
        0x75 => AccumulatorBinaryOp(add_with_carry, ZeroPageIndexed(registers.x_index)),

        // ROR zp,X
        0x76 => ReadModifyWrite(rotate_right, ZeroPageIndexed(registers.x_index)),

        // SEI
        0x78 => SetFlag(set_interrupt_disable, true),

        // ADC abs,Y
        0x79 => AccumulatorBinaryOp(add_with_carry, AbsoluteIndexed(registers.y_index)),

        // ADC abs,X
        0x7d => AccumulatorBinaryOp(add_with_carry, AbsoluteIndexed(registers.x_index)),

        // ROR abs,X
        0x7e => ReadModifyWrite(rotate_right, AbsoluteIndexed(registers.x_index)),

        // STA (zp,X)
        0x81 => Store(registers.accumulator, IndexedIndirect(registers.x_index)),

        // STY zp
        0x84 => Store(registers.y_index, ZeroPage),

        // STA zp
        0x85 => Store(registers.accumulator, ZeroPage),

        // STX zp
        0x86 => Store(registers.x_index, ZeroPage),

        // SAX zp
        0x87 => Store(registers.accumulator & registers.x_index, ZeroPage),

        // DEY
        0x88 => YIndexUnaryOp(decrement),

        // TXA
        0x8a => TransferRegister(registers.x_index, set_accumulator),

        // STY abs
        0x8c => Store(registers.y_index, Absolute),

        // STA abs
        0x8d => Store(registers.accumulator, Absolute),

        // STX abs
        0x8e => Store(registers.x_index, Absolute),

        // BCC rel
        0x90 => Branch(!registers.processor_flags.carry),

        // STA (zp),Y
        0x91 => Store(registers.accumulator, IndirectIndexed(registers.y_index)),

        // STY zp,X
        0x94 => Store(registers.y_index, ZeroPageIndexed(registers.x_index)),

        // STA zp,X
        0x95 => Store(registers.accumulator, ZeroPageIndexed(registers.x_index)),

        // STX zp,Y
        0x96 => Store(registers.x_index, ZeroPageIndexed(registers.y_index)),

        // STA abs,Y
        0x99 => Store(registers.accumulator, AbsoluteIndexed(registers.y_index)),

        // TYA
        0x98 => TransferRegister(registers.y_index, set_accumulator),

        // TXS
        0x9a => TransferRegisterNoFlags(registers.x_index, set_stack_pointer),

        // SHY abs,X
        0x9c => StoreHighAddressAndY(AbsoluteIndexed(registers.x_index)),

        // STA abs,X
        0x9d => Store(registers.accumulator, AbsoluteIndexed(registers.x_index)),

        // LDY imm
        0xa0 => Load(set_y_index, Immediate),

        // LDA (zp,X)
        0xa1 => Load(set_accumulator, IndexedIndirect(registers.x_index)),

        // LDX imm
        0xa2 => Load(set_x_index, Immediate),

        // LDY zp
        0xa4 => Load(set_y_index, ZeroPage),

        // LDA zp
        0xa5 => Load(set_accumulator, ZeroPage),

        // LDX zp
        0xa6 => Load(set_x_index, ZeroPage),

        // TAY
        0xa8 => TransferRegister(registers.accumulator, set_y_index),

        // LDA imm
        0xa9 => Load(set_accumulator, Immediate),

        // TXA
        0xaa => TransferRegister(registers.accumulator, set_x_index),

        // LDY abs
        0xac => Load(set_y_index, Absolute),

        // LDA abs
        0xad => Load(set_accumulator, Absolute),

        // LDX abs
        0xae => Load(set_x_index, Absolute),

        // BCS rel
        0xb0 => Branch(registers.processor_flags.carry),

        // LDA (zp),Y
        0xb1 => Load(set_accumulator, IndirectIndexed(registers.y_index)),

        // LDY zp,X
        0xb4 => Load(set_y_index, ZeroPageIndexed(registers.x_index)),

        // LDA zp,X
        0xb5 => Load(set_accumulator, ZeroPageIndexed(registers.x_index)),

        // LDX zp,Y
        0xb6 => Load(set_x_index, ZeroPageIndexed(registers.y_index)),

        // CLV
        0xb8 => SetFlag(set_overflow, false),

        // LDA abs,Y
        0xb9 => Load(set_accumulator, AbsoluteIndexed(registers.y_index)),

        // TSX
        0xba => TransferRegister(registers.stack_pointer, set_x_index),

        // LDY abs,X
        0xbc => Load(set_y_index, AbsoluteIndexed(registers.x_index)),

        // LDA abs,X
        0xbd => Load(set_accumulator, AbsoluteIndexed(registers.x_index)),

        // LDX abs,Y
        0xbe => Load(set_x_index, AbsoluteIndexed(registers.y_index)),

        // CPY imm
        0xc0 => Compare(registers.y_index, Immediate),

        // CMP (zp,X)
        0xc1 => Compare(registers.accumulator, IndexedIndirect(registers.x_index)),

        // CPY zp
        0xc4 => Compare(registers.y_index, ZeroPage),

        // CMP zp
        0xc5 => Compare(registers.accumulator, ZeroPage),

        // DEC zp
        0xc6 => ReadModifyWrite(decrement, ZeroPage),

        // INY
        0xc8 => YIndexUnaryOp(increment),

        // CMP abs
        0xc9 => Compare(registers.accumulator, Immediate),

        // DEX
        0xca => XIndexUnaryOp(decrement),

        // CPY abs
        0xcc => Compare(registers.y_index, Absolute),

        // CMP abs
        0xcd => Compare(registers.accumulator, Absolute),

        // DEC abs
        0xce => ReadModifyWrite(decrement, Absolute),

        // BNE rel
        0xd0 => Branch(!registers.processor_flags.zero),

        // CMP (zp),Y
        0xd1 => Compare(registers.accumulator, IndirectIndexed(registers.y_index)),

        // CMP zp,X
        0xd5 => Compare(registers.accumulator, ZeroPageIndexed(registers.x_index)),

        // DEC zp,X
        0xd6 => ReadModifyWrite(decrement, ZeroPageIndexed(registers.x_index)),

        // CLD
        0xd8 => SetFlag(set_decimal_mode, false),

        // CMP abs,Y
        0xd9 => Compare(registers.accumulator, AbsoluteIndexed(registers.y_index)),

        // NOP abs,X
        0xdc => NopRead(AbsoluteIndexed(registers.x_index)),

        // CMP abs,X
        0xdd => Compare(registers.accumulator, AbsoluteIndexed(registers.x_index)),

        // DEC abs,X
        0xde => ReadModifyWrite(decrement, AbsoluteIndexed(registers.x_index)),

        // CPX imm
        0xe0 => Compare(registers.x_index, Immediate),

        // SBC (zp,X)
        0xe1 => AccumulatorBinaryOp(subtract_with_carry, IndexedIndirect(registers.x_index)),

        // CPX zp
        0xe4 => Compare(registers.x_index, ZeroPage),

        // SBC zp
        0xe5 => AccumulatorBinaryOp(subtract_with_carry, ZeroPage),

        // INC zp
        0xe6 => ReadModifyWrite(increment, ZeroPage),

        // INX
        0xe8 => XIndexUnaryOp(increment),

        // SBC imm
        0xe9 => AccumulatorBinaryOp(subtract_with_carry, Immediate),

        // NOP
        0xea => Nop,

        // CPX abs
        0xec => Compare(registers.x_index, Absolute),

        // SBC abs
        0xed => AccumulatorBinaryOp(subtract_with_carry, Absolute),

        // INC abs
        0xee => ReadModifyWrite(increment, Absolute),

        // BEQ rel
        0xf0 => Branch(registers.processor_flags.zero),

        // SBC (zp),Y
        0xf1 => AccumulatorBinaryOp(subtract_with_carry, IndirectIndexed(registers.y_index)),

        // SBC zp,X
        0xf5 => AccumulatorBinaryOp(subtract_with_carry, ZeroPageIndexed(registers.x_index)),

        // INC zp,X
        0xf6 => ReadModifyWrite(increment, ZeroPageIndexed(registers.x_index)),

        // SED
        0xf8 => SetFlag(set_decimal_mode, true),

        // SBC abs,Y
        0xf9 => AccumulatorBinaryOp(subtract_with_carry, AbsoluteIndexed(registers.y_index)),

        // SBC abs,X
        0xfd => AccumulatorBinaryOp(subtract_with_carry, AbsoluteIndexed(registers.x_index)),

        // INC abs,X
        0xfe => ReadModifyWrite(increment, AbsoluteIndexed(registers.x_index)),

        _ => panic!("Unimplemented opcode: {:#04x}", opcode),
    }
}

impl Instruction {
    pub fn execute<T: BusTrait>(&self, bus: &mut T, registers: &mut Registers) {
        match self {
            NopRead(address_mode) => {
                address_mode.data(bus, &mut registers.program_counter);
            }

            Nop => {
                bus.phantom_read(registers.program_counter);
            }

            Store(value, address_mode) => {
                let address = address_mode.address(bus, &mut registers.program_counter);

                bus.write(address, *value, CycleOp::CheckInterrupt);
            }

            ReadModifyWrite(unary_op, address_mode) => {
                let address = address_mode.address(bus, &mut registers.program_counter);

                let old_value = bus.read(address, CycleOp::Sync);

                bus.write(address, old_value, CycleOp::Sync);

                let new_value = unary_op(&mut registers.processor_flags, old_value);

                bus.write(address, new_value, CycleOp::Sync);
            }

            ReadModifyWriteWithAccumulator(unary_op, binary_op, address_mode) => {
                let address = address_mode.address(bus, &mut registers.program_counter);

                let old_value = bus.read(address, CycleOp::Sync);

                bus.write(address, old_value, CycleOp::Sync);

                let new_value = unary_op(&mut registers.processor_flags, old_value);

                bus.write(address, new_value, CycleOp::Sync);

                registers.accumulator = binary_op(
                    &mut registers.processor_flags,
                    registers.accumulator,
                    new_value,
                );
            }

            AccumulatorUnaryOp(unary_op) => {
                bus.phantom_read(registers.program_counter);

                registers.accumulator =
                    unary_op(&mut registers.processor_flags, registers.accumulator);
            }

            XIndexUnaryOp(unary_op) => {
                bus.phantom_read(registers.program_counter);

                registers.x_index = unary_op(&mut registers.processor_flags, registers.x_index);
            }

            YIndexUnaryOp(unary_op) => {
                bus.phantom_read(registers.program_counter);

                registers.y_index = unary_op(&mut registers.processor_flags, registers.y_index);
            }

            AccumulatorBinaryOp(binary_op, address_mode) => {
                let operand = address_mode.data(bus, &mut registers.program_counter);

                registers.accumulator = binary_op(
                    &mut registers.processor_flags,
                    registers.accumulator,
                    operand,
                );
            }

            SetFlag(set_flag_fn, value) => {
                bus.phantom_read(registers.program_counter);

                set_flag_fn(&mut registers.processor_flags, *value);
            }

            Break(advance_return_address, additional_processor_flags, interrupt_vector) => {
                bus.phantom_read(registers.program_counter);

                if *advance_return_address {
                    advance_program_counter(&mut registers.program_counter);
                }

                push_16(bus, &mut registers.stack_pointer, registers.program_counter);

                push(
                    bus,
                    &mut registers.stack_pointer,
                    u8::from(registers.processor_flags) | additional_processor_flags,
                );

                registers.processor_flags.interrupt_disable = true;

                registers.program_counter = read_16(bus, *interrupt_vector, CycleOp::None);
            }

            JumpToSubRoutine => {
                let program_counter_low = read_immediate(bus, &mut registers.program_counter);

                phantom_stack_read(bus, registers.stack_pointer);

                push_16(bus, &mut registers.stack_pointer, registers.program_counter);

                let program_counter_high = read_immediate(bus, &mut registers.program_counter);

                registers.program_counter =
                    u16::from_le_bytes([program_counter_low, program_counter_high]);
            }

            Jump(address_mode) => {
                registers.program_counter =
                    address_mode.address(bus, &mut registers.program_counter);
            }

            ReturnFromInterrupt => {
                bus.phantom_read(registers.program_counter);

                phantom_stack_read(bus, registers.stack_pointer);

                registers.processor_flags = pop(bus, &mut registers.stack_pointer).into();

                registers.program_counter = pop_16(bus, &mut registers.stack_pointer);
            }

            ReturnFromSubroutine => {
                bus.phantom_read(registers.program_counter);

                phantom_stack_read(bus, registers.stack_pointer);

                registers.program_counter = pop_16(bus, &mut registers.stack_pointer);

                bus.phantom_read(registers.program_counter);

                advance_program_counter(&mut registers.program_counter);
            }

            PullAccumulator => {
                bus.phantom_read(registers.program_counter);

                phantom_stack_read(bus, registers.stack_pointer);

                registers.accumulator = pop(bus, &mut registers.stack_pointer);

                registers
                    .processor_flags
                    .update_zero_negative(registers.accumulator);
            }

            PushAccumulator => {
                bus.phantom_read(registers.program_counter);

                push(bus, &mut registers.stack_pointer, registers.accumulator);
            }

            PullProcessorFlags => {
                bus.phantom_read(registers.program_counter);

                phantom_stack_read(bus, registers.stack_pointer);

                registers.processor_flags = pop(bus, &mut registers.stack_pointer).into();
            }

            PushProcessorFlags => {
                bus.phantom_read(registers.program_counter);

                push(
                    bus,
                    &mut registers.stack_pointer,
                    u8::from(registers.processor_flags) | P_BREAK,
                );
            }

            Branch(condition) => {
                if !condition {
                    bus.phantom_read(registers.program_counter);

                    advance_program_counter(&mut registers.program_counter);
                } else {
                    registers.program_counter =
                        Relative.address(bus, &mut registers.program_counter);
                }
            }

            Compare(register_value, address_mode) => {
                let value = address_mode.data(bus, &mut registers.program_counter);

                registers.processor_flags.carry = *register_value >= value;
                registers.processor_flags.zero = *register_value == value;

                let diff = (register_value).wrapping_sub(value);
                registers.processor_flags.update_negative(diff);
            }

            Load(set_register, address_mode) => {
                let value = address_mode.data(bus, &mut registers.program_counter);

                set_register(registers, value);

                registers.processor_flags.update_zero_negative(value);
            }

            TransferRegister(value, set_register) => {
                bus.phantom_read(registers.program_counter);

                set_register(registers, *value);

                registers.processor_flags.update_zero_negative(*value);
            }

            TransferRegisterNoFlags(value, set_register) => {
                bus.phantom_read(registers.program_counter);

                set_register(registers, *value);
            }

            StoreHighAddressAndY(address_mode) => {
                let (address, carried) =
                    address_mode.address_with_carry(bus, &mut registers.program_counter);

                let [low, high] = address.to_le_bytes();

                if carried {
                    let value = registers.y_index & high;

                    let address = u16::from_le_bytes([low, value]);

                    bus.write(address, value, CycleOp::CheckInterrupt);
                } else {
                    let value = registers.y_index & high.wrapping_add(1);

                    bus.write(address, value, CycleOp::CheckInterrupt);
                };
            }
        }
    }
}

enum Instruction {
    Nop,
    NopRead(AddressMode),
    Store(u8, AddressMode),
    ReadModifyWrite(UnaryOp, AddressMode),
    ReadModifyWriteWithAccumulator(UnaryOp, BinaryOp, AddressMode),
    AccumulatorUnaryOp(UnaryOp),
    XIndexUnaryOp(UnaryOp),
    YIndexUnaryOp(UnaryOp),
    AccumulatorBinaryOp(BinaryOp, AddressMode),
    SetFlag(SetFlagFn, bool),
    Break(bool, u8, u16),
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
    Load(SetRegisterFn, AddressMode),
    TransferRegister(u8, SetRegisterFn),
    TransferRegisterNoFlags(u8, SetRegisterFn),
    StoreHighAddressAndY(AddressMode),
}

pub const NMI_VECTOR: u16 = 0xfffa;
pub const RESET_VECTOR: u16 = 0xfffc;
pub const IRQ_BRK_VECTOR: u16 = 0xfffe;

type SetRegisterFn = fn(registers: &mut Registers, u8);

fn set_stack_pointer(registers: &mut Registers, value: u8) {
    registers.stack_pointer = value;
}

fn set_accumulator(registers: &mut Registers, value: u8) {
    registers.accumulator = value;
}

fn set_x_index(registers: &mut Registers, value: u8) {
    registers.x_index = value;
}

fn set_y_index(registers: &mut Registers, value: u8) {
    registers.y_index = value;
}

type SetFlagFn = fn(&mut ProcessorFlags, bool);

fn set_carry(processor_flags: &mut ProcessorFlags, value: bool) {
    processor_flags.carry = value;
}

fn set_interrupt_disable(processor_flags: &mut ProcessorFlags, value: bool) {
    processor_flags.interrupt_disable = value;
}

fn set_decimal_mode(processor_flags: &mut ProcessorFlags, value: bool) {
    processor_flags.decimal_mode = value;
}

fn set_overflow(processor_flags: &mut ProcessorFlags, value: bool) {
    processor_flags.overflow = value;
}
