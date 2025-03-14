use crate::cycle_manager::*;

use super::registers::*;

mod addressing;
mod binary_ops;
mod memory_access;
mod unary_ops;

use addressing::*;
use binary_ops::*;
use memory_access::MemoryAccess;
use unary_ops::*;

use AddressMode::*;

pub struct Executor<'a, T: CycleManagerTrait + 'a> {
    memory_access: MemoryAccess<'a, T>,
    accumulator: &'a mut u8,
    x: &'a mut u8,
    y: &'a mut u8,
    processor_flags: &'a mut ProcessorFlags,
}

impl<'a, T: CycleManagerTrait + 'a> Executor<'a, T> {
    pub fn new(cycle_manager: &'a mut T, registers: &'a mut Registers) -> Self {
        Executor {
            memory_access: MemoryAccess::new(
                cycle_manager,
                &mut registers.program_counter,
                &mut registers.stack_pointer,
            ),
            accumulator: &mut registers.accumulator,
            x: &mut registers.x_index,
            y: &mut registers.y_index,
            processor_flags: &mut registers.processor_flags,
        }
    }

    pub fn interrupt(&mut self, nmi: bool) {
        self.memory_access.phantom_program_counter_read();

        self.break_inst(false, 0, if nmi { NMI_VECTOR } else { IRQ_BRK_VECTOR });

        self.memory_access.complete_instruction();
    }

    pub fn execute(&mut self, allow_untested_in_wild: bool) {
        let opcode = self.memory_access.read_immediate();

        if [0x35, 0x36, 0x41, 0x56, 0x5e, 0xe1].contains(&opcode) && !allow_untested_in_wild {
            panic!("untested opcode: {:02x}", opcode);
        }

        match opcode {
            // BRK
            0x00 => self.break_inst(true, P_BREAK, IRQ_BRK_VECTOR),

            // ORA (zp,X)
            0x01 => self.accumulator_binary_op(or, IndexedIndirect(*self.x)),

            // DOP zp
            0x04 => self.nop_read(ZeroPage),

            // ORA zp
            0x05 => self.accumulator_binary_op(or, ZeroPage),

            // ASL zp
            0x06 => self.read_modify_write(shift_left, ZeroPage),

            // SLO zp
            0x07 => self.read_modify_write_with_accumulator_op(shift_left, or, ZeroPage),

            // PHP
            0x08 => self.push_processor_flags(),

            // ORA imm
            0x09 => self.accumulator_binary_op(or, Immediate),

            // ASL A
            0x0a => self.accumulator_unary_op(shift_left),

            // ANC imm
            0x0b => self.accumulator_binary_op(and_negative_carry, Immediate),

            // ORA abs
            0x0d => self.accumulator_binary_op(or, Absolute),

            // ASL abs
            0x0e => self.read_modify_write(shift_left, Absolute),

            // BPL rel
            0x10 => self.branch(!self.processor_flags.negative),

            // ORA (zp),Y
            0x11 => self.accumulator_binary_op(or, IndirectIndexed(*self.y)),

            // ORA zp,X
            0x15 => self.accumulator_binary_op(or, ZeroPageIndexed(*self.x)),

            // ASL zp,X
            0x16 => self.read_modify_write(shift_left, ZeroPageIndexed(*self.x)),

            // CLC
            0x18 => self.processor_flags_op(|flags| flags.carry = false),

            // ORA abs,X
            0x1d => self.accumulator_binary_op(or, AbsoluteIndexed(*self.x)),

            // ASL abs,X
            0x1e => self.read_modify_write(shift_left, AbsoluteIndexed(*self.x)),

            // ORA abs,Y
            0x19 => self.accumulator_binary_op(or, AbsoluteIndexed(*self.y)),

            // JSR abs
            0x20 => self.jump_to_subroutine(),

            // AND (zp,X)
            0x21 => self.accumulator_binary_op(and, IndexedIndirect(*self.x)),

            // BIT zp
            0x24 => self.accumulator_binary_op(bit_test, ZeroPage),

            // AND zp
            0x25 => self.accumulator_binary_op(and, ZeroPage),

            // ROL zp
            0x26 => self.read_modify_write(rotate_left, ZeroPage),

            // PLP
            0x28 => self.pull_processor_flags(),

            // AND imm
            0x29 => self.accumulator_binary_op(and, Immediate),

            // ROL A
            0x2a => self.accumulator_unary_op(rotate_left),

            // BIT abs
            0x2c => self.accumulator_binary_op(bit_test, Absolute),

            // AND abs
            0x2d => self.accumulator_binary_op(and, Absolute),

            // ROL abs
            0x2e => self.read_modify_write(rotate_left, Absolute),

            // BMI rel
            0x30 => self.branch(self.processor_flags.negative),

            // AND (zp),Y
            0x31 => self.accumulator_binary_op(and, IndirectIndexed(*self.y)),

            // AND zp,X
            0x35 => self.accumulator_binary_op(and, ZeroPageIndexed(*self.x)),

            // ROL zp,X
            0x36 => self.read_modify_write(rotate_left, ZeroPageIndexed(*self.x)),

            // SEC
            0x38 => self.processor_flags_op(|flags| flags.carry = true),

            // AND abs,Y
            0x39 => self.accumulator_binary_op(and, AbsoluteIndexed(*self.y)),

            // AND abs,X
            0x3d => self.accumulator_binary_op(and, AbsoluteIndexed(*self.x)),

            // ROL abs,X
            0x3e => self.read_modify_write(rotate_left, AbsoluteIndexed(*self.x)),

            // RTI
            0x40 => self.return_from_interrupt(),

            // EOR (zp,X)
            0x41 => self.accumulator_binary_op(xor, IndexedIndirect(*self.x)),

            // EOR zp
            0x45 => self.accumulator_binary_op(xor, ZeroPage),

            // LSR zp
            0x46 => self.read_modify_write(shift_right, ZeroPage),

            // PHA
            0x48 => self.push_accumulator(),

            // EOR imm
            0x49 => self.accumulator_binary_op(xor, Immediate),

            // LSR A
            0x4a => self.accumulator_unary_op(shift_right),

            // ALR imm
            0x4b => self.accumulator_binary_op(and_shift_right, Immediate),

            // JMP abs
            0x4c => self.jump(Absolute),

            // EOR abs
            0x4d => self.accumulator_binary_op(xor, Absolute),

            // LSR abs
            0x4e => self.read_modify_write(shift_right, Absolute),

            // BVC rel
            0x50 => self.branch(!self.processor_flags.overflow),

            // EOR (zp),Y
            0x51 => self.accumulator_binary_op(xor, IndirectIndexed(*self.y)),

            // EOR zp,X
            0x55 => self.accumulator_binary_op(xor, ZeroPageIndexed(*self.x)),

            // LSR zp,X
            0x56 => self.read_modify_write(shift_right, ZeroPageIndexed(*self.x)),

            // CLI
            0x58 => self.processor_flags_op(|flags| flags.interrupt_disable = false),

            // EOR abs,Y
            0x59 => self.accumulator_binary_op(xor, AbsoluteIndexed(*self.y)),

            // EOR abs,X
            0x5d => self.accumulator_binary_op(xor, AbsoluteIndexed(*self.x)),

            // LSR abs,X
            0x5e => self.read_modify_write(shift_right, AbsoluteIndexed(*self.x)),

            // RTS
            0x60 => self.return_from_subroutine(),

            // ADC (zp,X)
            0x61 => self.accumulator_binary_op(add_with_carry, IndexedIndirect(*self.x)),

            // ADC zp
            0x65 => self.accumulator_binary_op(add_with_carry, ZeroPage),

            // ROR zp
            0x66 => self.read_modify_write(rotate_right, ZeroPage),

            // PLA
            0x68 => self.pull_accumulator(),

            // ADC imm
            0x69 => self.accumulator_binary_op(add_with_carry, Immediate),

            // ROR A
            0x6a => self.accumulator_unary_op(rotate_right),

            // JMP (abs)
            0x6c => self.jump(Indirect),

            // ADC abs
            0x6d => self.accumulator_binary_op(add_with_carry, Absolute),

            // ROR abs
            0x6e => self.read_modify_write(rotate_right, Absolute),

            // BVS rel
            0x70 => self.branch(self.processor_flags.overflow),

            // ADC (zp)
            0x71 => self.accumulator_binary_op(add_with_carry, IndirectIndexed(*self.y)),

            // ADC zp,X
            0x75 => self.accumulator_binary_op(add_with_carry, ZeroPageIndexed(*self.x)),

            // ROR zp,X
            0x76 => self.read_modify_write(rotate_right, ZeroPageIndexed(*self.x)),

            // SEI
            0x78 => self.processor_flags_op(|flags| flags.interrupt_disable = true),

            // ADC abs,Y
            0x79 => self.accumulator_binary_op(add_with_carry, AbsoluteIndexed(*self.y)),

            // ADC abs,X
            0x7d => self.accumulator_binary_op(add_with_carry, AbsoluteIndexed(*self.x)),

            // ROR abs,X
            0x7e => self.read_modify_write(rotate_right, AbsoluteIndexed(*self.x)),

            // STA (zp,X)
            0x81 => self.store(*self.accumulator, IndexedIndirect(*self.x)),

            // STY zp
            0x84 => self.store(*self.y, ZeroPage),

            // STA zp
            0x85 => self.store(*self.accumulator, ZeroPage),

            // STX zp
            0x86 => self.store(*self.x, ZeroPage),

            // SAX zp
            0x87 => self.store(*self.accumulator & *self.x, ZeroPage),

            // DEY
            0x88 => self.y_index_unary_op(decrement),

            // TXA
            0x8a => self.transfer_register(*self.x, set_accumulator),

            // STY abs
            0x8c => self.store(*self.y, Absolute),

            // STA abs
            0x8d => self.store(*self.accumulator, Absolute),

            // STX abs
            0x8e => self.store(*self.x, Absolute),

            // BCC rel
            0x90 => self.branch(!self.processor_flags.carry),

            // STA (zp),Y
            0x91 => self.store(*self.accumulator, IndirectIndexed(*self.y)),

            // STY zp,X
            0x94 => self.store(*self.y, ZeroPageIndexed(*self.x)),

            // STA zp,X
            0x95 => self.store(*self.accumulator, ZeroPageIndexed(*self.x)),

            // STX zp,Y
            0x96 => self.store(*self.x, ZeroPageIndexed(*self.y)),

            // STA abs,Y
            0x99 => self.store(*self.accumulator, AbsoluteIndexed(*self.y)),

            // TYA
            0x98 => self.transfer_register(*self.y, set_accumulator),

            // TXS
            0x9a => self.transfer_register_no_flags(*self.x, set_stack_pointer),

            // SHY abs,X
            0x9c => self.store_high_address_and_y(AbsoluteIndexed(*self.x)),

            // STA abs,X
            0x9d => self.store(*self.accumulator, AbsoluteIndexed(*self.x)),

            // LDY imm
            0xa0 => self.load_register(set_y_index, Immediate),

            // LDA (zp,X)
            0xa1 => self.load_register(set_accumulator, IndexedIndirect(*self.x)),

            // LDX imm
            0xa2 => self.load_register(set_x_index, Immediate),

            // LDY zp
            0xa4 => self.load_register(set_y_index, ZeroPage),

            // LDA zp
            0xa5 => self.load_register(set_accumulator, ZeroPage),

            // LDX zp
            0xa6 => self.load_register(set_x_index, ZeroPage),

            // TAY
            0xa8 => self.transfer_register(*self.accumulator, set_y_index),

            // LDA imm
            0xa9 => self.load_register(set_accumulator, Immediate),

            // TXA
            0xaa => self.transfer_register(*self.accumulator, set_x_index),

            // LDY abs
            0xac => self.load_register(set_y_index, Absolute),

            // LDA abs
            0xad => self.load_register(set_accumulator, Absolute),

            // LDX abs
            0xae => self.load_register(set_x_index, Absolute),

            // BCS rel
            0xb0 => self.branch(self.processor_flags.carry),

            // LDA (zp),Y
            0xb1 => self.load_register(set_accumulator, IndirectIndexed(*self.y)),

            // LDY zp,X
            0xb4 => self.load_register(set_y_index, ZeroPageIndexed(*self.x)),

            // LDA zp,X
            0xb5 => self.load_register(set_accumulator, ZeroPageIndexed(*self.x)),

            // LDX zp,Y
            0xb6 => self.load_register(set_x_index, ZeroPageIndexed(*self.y)),

            // CLV
            0xb8 => self.processor_flags_op(|flags| flags.overflow = false),

            // LDA abs,Y
            0xb9 => self.load_register(set_accumulator, AbsoluteIndexed(*self.y)),

            // TSX
            0xba => self.transfer_register(*self.memory_access.stack_pointer, set_x_index),

            // LDY abs,X
            0xbc => self.load_register(set_y_index, AbsoluteIndexed(*self.x)),

            // LDA abs,X
            0xbd => self.load_register(set_accumulator, AbsoluteIndexed(*self.x)),

            // LDX abs,Y
            0xbe => self.load_register(set_x_index, AbsoluteIndexed(*self.y)),

            // CPY imm
            0xc0 => self.compare(*self.y, Immediate),

            // CMP (zp,X)
            0xc1 => self.compare(*self.accumulator, IndexedIndirect(*self.x)),

            // CPY zp
            0xc4 => self.compare(*self.y, ZeroPage),

            // CMP zp
            0xc5 => self.compare(*self.accumulator, ZeroPage),

            // DEC zp
            0xc6 => self.read_modify_write(decrement, ZeroPage),

            // INY
            0xc8 => self.y_index_unary_op(increment),

            // CMP abs
            0xc9 => self.compare(*self.accumulator, Immediate),

            // DEX
            0xca => self.x_index_unary_op(decrement),

            // CPY abs
            0xcc => self.compare(*self.y, Absolute),

            // CMP abs
            0xcd => self.compare(*self.accumulator, Absolute),

            // DEC abs
            0xce => self.read_modify_write(decrement, Absolute),

            // BNE rel
            0xd0 => self.branch(!self.processor_flags.zero),

            // CMP (zp),Y
            0xd1 => self.compare(*self.accumulator, IndirectIndexed(*self.y)),

            // CMP zp,X
            0xd5 => self.compare(*self.accumulator, ZeroPageIndexed(*self.x)),

            // DEC zp,X
            0xd6 => self.read_modify_write(decrement, ZeroPageIndexed(*self.x)),

            // CLD
            0xd8 => self.processor_flags_op(|flags| flags.decimal_mode = false),

            // CMP abs,Y
            0xd9 => self.compare(*self.accumulator, AbsoluteIndexed(*self.y)),

            // NOP abs,X
            0xdc => self.nop_read(AbsoluteIndexed(*self.x)),

            // CMP abs,X
            0xdd => self.compare(*self.accumulator, AbsoluteIndexed(*self.x)),

            // DEC abs,X
            0xde => self.read_modify_write(decrement, AbsoluteIndexed(*self.x)),

            // CPX imm
            0xe0 => self.compare(*self.x, Immediate),

            // SBC (zp,X)
            0xe1 => self.accumulator_binary_op(subtract_with_carry, IndexedIndirect(*self.x)),

            // CPX zp
            0xe4 => self.compare(*self.x, ZeroPage),

            // SBC zp
            0xe5 => self.accumulator_binary_op(subtract_with_carry, ZeroPage),

            // INC zp
            0xe6 => self.read_modify_write(increment, ZeroPage),

            // INX
            0xe8 => self.x_index_unary_op(increment),

            // SBC imm
            0xe9 => self.accumulator_binary_op(subtract_with_carry, Immediate),

            // NOP
            0xea => self.memory_access.phantom_program_counter_read(),

            // CPX abs
            0xec => self.compare(*self.x, Absolute),

            // SBC abs
            0xed => self.accumulator_binary_op(subtract_with_carry, Absolute),

            // INC abs
            0xee => self.read_modify_write(increment, Absolute),

            // BEQ rel
            0xf0 => self.branch(self.processor_flags.zero),

            // SBC (zp),Y
            0xf1 => self.accumulator_binary_op(subtract_with_carry, IndirectIndexed(*self.y)),

            // SBC zp,X
            0xf5 => self.accumulator_binary_op(subtract_with_carry, ZeroPageIndexed(*self.x)),

            // INC zp,X
            0xf6 => self.read_modify_write(increment, ZeroPageIndexed(*self.x)),

            // SED
            0xf8 => self.processor_flags_op(|flags| flags.decimal_mode = true),

            // SBC abs,Y
            0xf9 => self.accumulator_binary_op(subtract_with_carry, AbsoluteIndexed(*self.y)),

            // SBC abs,X
            0xfd => self.accumulator_binary_op(subtract_with_carry, AbsoluteIndexed(*self.x)),

            // INC abs,X
            0xfe => self.read_modify_write(increment, AbsoluteIndexed(*self.x)),

            _ => panic!("Unimplemented opcode: {:#04x}", opcode),
        }

        self.memory_access.complete_instruction();
    }

    fn address(&mut self, address_mode: AddressMode) -> u16 {
        address_mode.address(&mut self.memory_access)
    }

    fn address_with_carry(&mut self, address_mode: AddressMode) -> (u16, bool) {
        address_mode.address_with_carry(&mut self.memory_access)
    }

    fn data(&mut self, address_mode: AddressMode) -> u8 {
        address_mode.data(&mut self.memory_access)
    }

    fn nop_read(&mut self, address_mode: AddressMode) {
        self.data(address_mode);
    }

    fn store(&mut self, value: u8, address_mode: AddressMode) {
        let address = self.address(address_mode);

        self.memory_access
            .write(address, value, CycleOp::CheckInterrupt);
    }

    fn read_modify_write(
        &mut self,
        unary_op: fn(&mut ProcessorFlags, u8) -> u8,
        address_mode: AddressMode,
    ) {
        self.read_modify_write_return_value(unary_op, address_mode);
    }

    fn read_modify_write_with_accumulator_op(
        &mut self,
        unary_op: fn(&mut ProcessorFlags, u8) -> u8,
        accumulator_op: fn(&mut ProcessorFlags, u8, u8) -> u8,
        address_mode: AddressMode,
    ) {
        let new_value = self.read_modify_write_return_value(unary_op, address_mode);

        *self.accumulator = accumulator_op(self.processor_flags, *self.accumulator, new_value);
    }

    fn read_modify_write_return_value(
        &mut self,
        unary_op: fn(&mut ProcessorFlags, u8) -> u8,
        address_mode: AddressMode,
    ) -> u8 {
        let address = self.address(address_mode);

        let old_value = self.memory_access.read(address, CycleOp::Sync);

        self.memory_access.write(address, old_value, CycleOp::Sync);

        let new_value = unary_op(self.processor_flags, old_value);

        self.memory_access.write(address, new_value, CycleOp::Sync);

        new_value
    }

    fn accumulator_unary_op(&mut self, unary_op: fn(&mut ProcessorFlags, u8) -> u8) {
        self.memory_access.phantom_program_counter_read();

        *self.accumulator = unary_op(self.processor_flags, *self.accumulator);
    }

    fn x_index_unary_op(&mut self, unary_op: fn(&mut ProcessorFlags, u8) -> u8) {
        self.memory_access.phantom_program_counter_read();

        *self.x = unary_op(self.processor_flags, *self.x);
    }

    fn y_index_unary_op(&mut self, unary_op: fn(&mut ProcessorFlags, u8) -> u8) {
        self.memory_access.phantom_program_counter_read();

        *self.y = unary_op(self.processor_flags, *self.y);
    }

    fn accumulator_binary_op(
        &mut self,
        binary_op: fn(&mut ProcessorFlags, u8, u8) -> u8,
        address_mode: AddressMode,
    ) {
        let operand = self.data(address_mode);

        *self.accumulator = binary_op(self.processor_flags, *self.accumulator, operand);
    }

    fn processor_flags_op<F: Fn(&mut ProcessorFlags)>(&mut self, callback: F) {
        self.memory_access.phantom_program_counter_read();

        callback(self.processor_flags);
    }

    fn break_inst(
        &mut self,
        advance_return_address: bool,
        additional_processor_flags: u8,
        interrupt_vector: u16,
    ) {
        self.memory_access.phantom_program_counter_read();

        if advance_return_address {
            self.memory_access.increment_program_counter();
        }

        self.memory_access.push_program_counter();

        self.memory_access
            .push(u8::from(*self.processor_flags) | additional_processor_flags);

        self.processor_flags.interrupt_disable = true;

        *self.memory_access.program_counter =
            self.memory_access.read_16(interrupt_vector, CycleOp::None);
    }

    fn jump_to_subroutine(&mut self) {
        let program_counter_low = self.memory_access.read_immediate();

        self.memory_access.phantom_stack_read();

        self.memory_access.push_program_counter();

        let program_counter_high = self.memory_access.read_immediate();

        *self.memory_access.program_counter =
            u16::from_le_bytes([program_counter_low, program_counter_high]);
    }

    fn jump(&mut self, address_mode: AddressMode) {
        *self.memory_access.program_counter = self.address(address_mode);
    }

    fn return_from_interrupt(&mut self) {
        self.memory_access.phantom_program_counter_read();

        self.memory_access.phantom_stack_read();

        *self.processor_flags = self.memory_access.pop().into();

        self.memory_access.pop_program_counter();
    }

    fn return_from_subroutine(&mut self) {
        self.memory_access.phantom_program_counter_read();

        self.memory_access.phantom_stack_read();

        self.memory_access.pop_program_counter();

        self.memory_access.phantom_program_counter_read();

        self.memory_access.increment_program_counter();
    }

    fn pull_accumulator(&mut self) {
        self.memory_access.phantom_program_counter_read();

        self.memory_access.phantom_stack_read();

        *self.accumulator = self.memory_access.pop();

        self.processor_flags.update_zero_negative(*self.accumulator);
    }

    fn push_accumulator(&mut self) {
        self.memory_access.phantom_program_counter_read();

        self.memory_access.push(*self.accumulator);
    }

    fn pull_processor_flags(&mut self) {
        self.memory_access.phantom_program_counter_read();

        self.memory_access.phantom_stack_read();

        *self.processor_flags = self.memory_access.pop().into();
    }

    fn push_processor_flags(&mut self) {
        self.memory_access.phantom_program_counter_read();

        self.memory_access
            .push(u8::from(*self.processor_flags) | P_BREAK);
    }

    fn branch(&mut self, condition: bool) {
        if !condition {
            self.memory_access.phantom_program_counter_read();

            self.memory_access.increment_program_counter();
        } else {
            *self.memory_access.program_counter = self.address(Relative);
        }
    }

    fn compare(&mut self, register_value: u8, address_mode: AddressMode) {
        let value = self.data(address_mode);

        self.processor_flags.carry = register_value >= value;
        self.processor_flags.zero = register_value == value;

        let diff = (register_value).wrapping_sub(value);
        self.processor_flags.update_negative(diff);
    }

    fn load_register(&mut self, set_register: fn(&mut Self, u8), address_mode: AddressMode) {
        let value = self.data(address_mode);

        set_register(self, value);

        self.processor_flags.update_zero_negative(value);
    }

    fn transfer_register(&mut self, value: u8, set_register: fn(&mut Self, u8)) {
        self.memory_access.phantom_program_counter_read();

        set_register(self, value);

        self.processor_flags.update_zero_negative(value);
    }

    fn transfer_register_no_flags(&mut self, value: u8, set_register: fn(&mut Self, u8)) {
        self.memory_access.phantom_program_counter_read();

        set_register(self, value);
    }

    fn store_high_address_and_y(&mut self, address_mode: AddressMode) {
        let (address, carried) = self.address_with_carry(address_mode);

        let [low, high] = address.to_le_bytes();

        if carried {
            let value = *self.y & high;

            let address = u16::from_le_bytes([low, value]);

            self.memory_access
                .write(address, value, CycleOp::CheckInterrupt);
        } else {
            let value = *self.y & high.wrapping_add(1);

            self.memory_access
                .write(address, value, CycleOp::CheckInterrupt);
        };
    }
}

pub const NMI_VECTOR: u16 = 0xfffa;
pub const RESET_VECTOR: u16 = 0xfffc;
pub const IRQ_BRK_VECTOR: u16 = 0xfffe;

fn set_stack_pointer<T: CycleManagerTrait>(executor: &mut Executor<T>, value: u8) {
    *executor.memory_access.stack_pointer = value;
}

fn set_accumulator<T: CycleManagerTrait>(executor: &mut Executor<T>, value: u8) {
    *executor.accumulator = value;
}

fn set_x_index<T: CycleManagerTrait>(executor: &mut Executor<T>, value: u8) {
    *executor.x = value;
}

fn set_y_index<T: CycleManagerTrait>(executor: &mut Executor<T>, value: u8) {
    *executor.y = value;
}
