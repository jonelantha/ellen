use core::panic;

use crate::cycle_manager::*;

use super::registers::*;

mod addressing;
mod binary_ops;
mod unary_ops;

use addressing::*;
use binary_ops::*;
use unary_ops::*;

pub struct Executor<'a, T: CycleManagerTrait + 'a> {
    cycle_manager: &'a mut T,
    registers: &'a mut Registers,
}

impl<'a, T: CycleManagerTrait + 'a> Executor<'a, T> {
    pub fn new(cycle_manager: &'a mut T, registers: &'a mut Registers) -> Self {
        Executor {
            cycle_manager,
            registers,
        }
    }

    pub fn interrupt(&mut self, nmi: bool) {
        self.phantom_program_counter_read();

        self.break_inst(0, 0, if nmi { NMI_VECTOR } else { IRQ_BRK_VECTOR });

        self.cycle_manager.complete();
    }

    pub fn execute(&mut self, allow_untested_in_wild: bool) {
        let opcode = self.imm();

        if [0x35, 0x36, 0x41, 0x56, 0x5e, 0xe1].contains(&opcode) && !allow_untested_in_wild {
            panic!("untested opcode: {:02x}", opcode);
        }

        match opcode {
            // BRK
            0x00 => self.break_inst(1, P_BREAK, IRQ_BRK_VECTOR),

            // ORA (zp,X)
            0x01 => self.accumulator_binary_op(or, indexed_indirect_x_data),

            // DOP zp
            0x04 => self.nop_read(zero_page_data),

            // ORA zp
            0x05 => self.accumulator_binary_op(or, zero_page_data),

            // ASL zp
            0x06 => self.read_modify_write(shift_left, zero_page_address),

            // SLO zp
            0x07 => self.read_modify_write_with_accumulator_op(shift_left, or, zero_page_address),

            // PHP
            0x08 => self.push_processor_flags(),

            // ORA imm
            0x09 => self.accumulator_binary_op(or, immediate_data),

            // ASL A
            0x0a => self.accumulator_unary_op(shift_left),

            // ANC imm
            0x0b => self.accumulator_binary_op(and_negative_carry, immediate_data),

            // ORA abs
            0x0d => self.accumulator_binary_op(or, absolute_data),

            // ASL abs
            0x0e => self.read_modify_write(shift_left, absolute_address),

            // BPL rel
            0x10 => self.branch(!self.registers.processor_flags.negative),

            // ORA (zp),Y
            0x11 => self.accumulator_binary_op(or, indirect_indexed_y_data),

            // ORA zp,X
            0x15 => self.accumulator_binary_op(or, zero_page_x_data),

            // ASL zp,X
            0x16 => self.read_modify_write(shift_left, zero_page_x_address),

            // CLC
            0x18 => self.processor_flags_op(|flags| flags.carry = false),

            // ORA abs,X
            0x1d => self.accumulator_binary_op(or, absolute_x_data),

            // ASL abs,X
            0x1e => self.read_modify_write(shift_left, absolute_x_address),

            // ORA abs,Y
            0x19 => self.accumulator_binary_op(or, absolute_y_data),

            // JSR abs
            0x20 => self.jump_to_subroutine(),

            // AND (zp,X)
            0x21 => self.accumulator_binary_op(and, indexed_indirect_x_data),

            // BIT zp
            0x24 => self.accumulator_binary_op(bit_test, zero_page_data),

            // AND zp
            0x25 => self.accumulator_binary_op(and, zero_page_data),

            // ROL zp
            0x26 => self.read_modify_write(rotate_left, zero_page_address),

            // PLP
            0x28 => self.pull_processor_flags(),

            // AND imm
            0x29 => self.accumulator_binary_op(and, immediate_data),

            // ROL A
            0x2a => self.accumulator_unary_op(rotate_left),

            // BIT abs
            0x2c => self.accumulator_binary_op(bit_test, absolute_data),

            // AND abs
            0x2d => self.accumulator_binary_op(and, absolute_data),

            // ROL abs
            0x2e => self.read_modify_write(rotate_left, absolute_address),

            // BMI rel
            0x30 => self.branch(self.registers.processor_flags.negative),

            // AND (zp),Y
            0x31 => self.accumulator_binary_op(and, indirect_indexed_y_data),

            // AND zp,X
            0x35 => self.accumulator_binary_op(and, zero_page_x_data),

            // ROL zp,X
            0x36 => self.read_modify_write(rotate_left, zero_page_x_address),

            // SEC
            0x38 => self.processor_flags_op(|flags| flags.carry = true),

            // AND abs,Y
            0x39 => self.accumulator_binary_op(and, absolute_y_data),

            // AND abs,X
            0x3d => self.accumulator_binary_op(and, absolute_x_data),

            // ROL abs,X
            0x3e => self.read_modify_write(rotate_left, absolute_x_address),

            // RTI
            0x40 => self.return_from_interrupt(),

            // EOR (zp,X)
            0x41 => self.accumulator_binary_op(xor, indexed_indirect_x_data),

            // EOR zp
            0x45 => self.accumulator_binary_op(xor, zero_page_data),

            // LSR zp
            0x46 => self.read_modify_write(shift_right, zero_page_address),

            // PHA
            0x48 => self.push_accumulator(),

            // EOR imm
            0x49 => self.accumulator_binary_op(xor, immediate_data),

            // LSR A
            0x4a => self.accumulator_unary_op(shift_right),

            // ALR imm
            0x4b => self.accumulator_binary_op(and_shift_right, immediate_data),

            // JMP abs
            0x4c => self.jump(absolute_address),

            // EOR abs
            0x4d => self.accumulator_binary_op(xor, absolute_data),

            // LSR abs
            0x4e => self.read_modify_write(shift_right, absolute_address),

            // BVC rel
            0x50 => self.branch(!self.registers.processor_flags.overflow),

            // EOR (zp),Y
            0x51 => self.accumulator_binary_op(xor, indirect_indexed_y_data),

            // EOR zp,X
            0x55 => self.accumulator_binary_op(xor, zero_page_x_data),

            // LSR zp,X
            0x56 => self.read_modify_write(shift_right, zero_page_x_address),

            // CLI
            0x58 => self.processor_flags_op(|flags| flags.interrupt_disable = false),

            // EOR abs,Y
            0x59 => self.accumulator_binary_op(xor, absolute_y_data),

            // EOR abs,X
            0x5d => self.accumulator_binary_op(xor, absolute_x_data),

            // LSR abs,X
            0x5e => self.read_modify_write(shift_right, absolute_x_address),

            // RTS
            0x60 => self.return_from_subroutine(),

            // ADC (zp,X)
            0x61 => self.accumulator_binary_op(add_with_carry, indexed_indirect_x_data),

            // ADC zp
            0x65 => self.accumulator_binary_op(add_with_carry, zero_page_data),

            // ROR zp
            0x66 => self.read_modify_write(rotate_right, zero_page_address),

            // PLA
            0x68 => self.pull_accumulator(),

            // ADC imm
            0x69 => self.accumulator_binary_op(add_with_carry, immediate_data),

            // ROR A
            0x6a => self.accumulator_unary_op(rotate_right),

            // JMP (abs)
            0x6c => self.jump(indirect_address),

            // ADC abs
            0x6d => self.accumulator_binary_op(add_with_carry, absolute_data),

            // ROR abs
            0x6e => self.read_modify_write(rotate_right, absolute_address),

            // BVS rel
            0x70 => self.branch(self.registers.processor_flags.overflow),

            // ADC (zp)
            0x71 => self.accumulator_binary_op(add_with_carry, indirect_indexed_y_data),

            // ADC zp,X
            0x75 => self.accumulator_binary_op(add_with_carry, zero_page_x_data),

            // ROR zp,X
            0x76 => self.read_modify_write(rotate_right, zero_page_x_address),

            // SEI
            0x78 => self.processor_flags_op(|flags| flags.interrupt_disable = true),

            // ADC abs,Y
            0x79 => self.accumulator_binary_op(add_with_carry, absolute_y_data),

            // ADC abs,X
            0x7d => self.accumulator_binary_op(add_with_carry, absolute_x_data),

            // ROR abs,X
            0x7e => self.read_modify_write(rotate_right, absolute_x_address),

            // STA (zp,X)
            0x81 => self.store(self.registers.accumulator, indexed_indirect_x_address),

            // STY zp
            0x84 => self.store(self.registers.y_index, zero_page_address),

            // STA zp
            0x85 => self.store(self.registers.accumulator, zero_page_address),

            // STX zp
            0x86 => self.store(self.registers.x_index, zero_page_address),

            // SAX zp
            0x87 => self.store(
                self.registers.accumulator & self.registers.x_index,
                zero_page_address,
            ),

            // DEY
            0x88 => self.y_index_unary_op(decrement),

            // TXA
            0x8a => self.transfer_register(self.registers.x_index, set_accumulator),

            // STY abs
            0x8c => self.store(self.registers.y_index, absolute_address),

            // STA abs
            0x8d => self.store(self.registers.accumulator, absolute_address),

            // STX abs
            0x8e => self.store(self.registers.x_index, absolute_address),

            // BCC rel
            0x90 => self.branch(!self.registers.processor_flags.carry),

            // STA (zp),Y
            0x91 => self.store(self.registers.accumulator, indirect_indexed_y_address),

            // STY zp,X
            0x94 => self.store(self.registers.y_index, zero_page_x_address),

            // STA zp,X
            0x95 => self.store(self.registers.accumulator, zero_page_x_address),

            // STX zp,Y
            0x96 => self.store(self.registers.x_index, zero_page_y_address),

            // STA abs,Y
            0x99 => self.store(self.registers.accumulator, absolute_y_address),

            // TYA
            0x98 => self.transfer_register(self.registers.y_index, set_accumulator),

            // TXS
            0x9a => self.transfer_register_no_flags(self.registers.x_index, set_stack_pointer),

            // SHY abs,X
            0x9c => self.store_high_address_and_y(absolute_x_address_with_carry),

            // STA abs,X
            0x9d => self.store(self.registers.accumulator, absolute_x_address),

            // LDY imm
            0xa0 => self.load_register(set_y_index, immediate_data),

            // LDA (zp,X)
            0xa1 => self.load_register(set_accumulator, indexed_indirect_x_data),

            // LDX imm
            0xa2 => self.load_register(set_x_index, immediate_data),

            // LDY zp
            0xa4 => self.load_register(set_y_index, zero_page_data),

            // LDA zp
            0xa5 => self.load_register(set_accumulator, zero_page_data),

            // LDX zp
            0xa6 => self.load_register(set_x_index, zero_page_data),

            // TAY
            0xa8 => self.transfer_register(self.registers.accumulator, set_y_index),

            // LDA imm
            0xa9 => self.load_register(set_accumulator, immediate_data),

            // TXA
            0xaa => self.transfer_register(self.registers.accumulator, set_x_index),

            // LDY abs
            0xac => self.load_register(set_y_index, absolute_data),

            // LDA abs
            0xad => self.load_register(set_accumulator, absolute_data),

            // LDX abs
            0xae => self.load_register(set_x_index, absolute_data),

            // BCS rel
            0xb0 => self.branch(self.registers.processor_flags.carry),

            // LDA (zp),Y
            0xb1 => self.load_register(set_accumulator, indirect_indexed_y_data),

            // LDY zp,X
            0xb4 => self.load_register(set_y_index, zero_page_x_data),

            // LDA zp,X
            0xb5 => self.load_register(set_accumulator, zero_page_x_data),

            // LDX zp,Y
            0xb6 => self.load_register(set_x_index, zero_page_y_data),

            // CLV
            0xb8 => self.processor_flags_op(|flags| flags.overflow = false),

            // LDA abs,Y
            0xb9 => self.load_register(set_accumulator, absolute_y_data),

            // TSX
            0xba => self.transfer_register(self.registers.stack_pointer, set_x_index),

            // LDY abs,X
            0xbc => self.load_register(set_y_index, absolute_x_data),

            // LDA abs,X
            0xbd => self.load_register(set_accumulator, absolute_x_data),

            // LDX abs,Y
            0xbe => self.load_register(set_x_index, absolute_y_data),

            // CPY imm
            0xc0 => self.compare(self.registers.y_index, immediate_data),

            // CMP (zp,X)
            0xc1 => self.compare(self.registers.accumulator, indexed_indirect_x_data),

            // CPY zp
            0xc4 => self.compare(self.registers.y_index, zero_page_data),

            // CMP zp
            0xc5 => self.compare(self.registers.accumulator, zero_page_data),

            // DEC zp
            0xc6 => self.read_modify_write(decrement, zero_page_address),

            // INY
            0xc8 => self.y_index_unary_op(increment),

            // CMP abs
            0xc9 => self.compare(self.registers.accumulator, immediate_data),

            // DEX
            0xca => self.x_index_unary_op(decrement),

            // CPY abs
            0xcc => self.compare(self.registers.y_index, absolute_data),

            // CMP abs
            0xcd => self.compare(self.registers.accumulator, absolute_data),

            // DEC abs
            0xce => self.read_modify_write(decrement, absolute_address),

            // BNE rel
            0xd0 => self.branch(!self.registers.processor_flags.zero),

            // CMP (zp),Y
            0xd1 => self.compare(self.registers.accumulator, indirect_indexed_y_data),

            // CMP zp,X
            0xd5 => self.compare(self.registers.accumulator, zero_page_x_data),

            // DEC zp,X
            0xd6 => self.read_modify_write(decrement, zero_page_x_address),

            // CLD
            0xd8 => self.processor_flags_op(|flags| flags.decimal_mode = false),

            // CMP abs,Y
            0xd9 => self.compare(self.registers.accumulator, absolute_y_data),

            // NOP abs,X
            0xdc => self.nop_read(absolute_x_data),

            // CMP abs,X
            0xdd => self.compare(self.registers.accumulator, absolute_x_data),

            // DEC abs,X
            0xde => self.read_modify_write(decrement, absolute_x_address),

            // CPX imm
            0xe0 => self.compare(self.registers.x_index, immediate_data),

            // SBC (zp,X)
            0xe1 => self.accumulator_binary_op(subtract_with_carry, indexed_indirect_x_data),

            // CPX zp
            0xe4 => self.compare(self.registers.x_index, zero_page_data),

            // SBC zp
            0xe5 => self.accumulator_binary_op(subtract_with_carry, zero_page_data),

            // INC zp
            0xe6 => self.read_modify_write(increment, zero_page_address),

            // INX
            0xe8 => self.x_index_unary_op(increment),

            // SBC imm
            0xe9 => self.accumulator_binary_op(subtract_with_carry, immediate_data),

            // NOP
            0xea => self.phantom_program_counter_read(),

            // CPX abs
            0xec => self.compare(self.registers.x_index, absolute_data),

            // SBC abs
            0xed => self.accumulator_binary_op(subtract_with_carry, absolute_data),

            // INC abs
            0xee => self.read_modify_write(increment, absolute_address),

            // BEQ rel
            0xf0 => self.branch(self.registers.processor_flags.zero),

            // SBC (zp),Y
            0xf1 => self.accumulator_binary_op(subtract_with_carry, indirect_indexed_y_data),

            // SBC zp,X
            0xf5 => self.accumulator_binary_op(subtract_with_carry, zero_page_x_data),

            // INC zp,X
            0xf6 => self.read_modify_write(increment, zero_page_x_address),

            // SED
            0xf8 => self.processor_flags_op(|flags| flags.decimal_mode = true),

            // SBC abs,Y
            0xf9 => self.accumulator_binary_op(subtract_with_carry, absolute_y_data),

            // SBC abs,X
            0xfd => self.accumulator_binary_op(subtract_with_carry, absolute_x_data),

            // INC abs,X
            0xfe => self.read_modify_write(increment, absolute_x_address),

            _ => panic!("Unimplemented opcode: {:#04x}", opcode),
        }

        self.cycle_manager.complete();
    }

    fn phantom_read(&mut self, address: u16) {
        self.cycle_manager.phantom_read(address);
    }

    fn phantom_program_counter_read(&mut self) {
        self.phantom_read(self.registers.program_counter);
    }

    fn read(&mut self, address: u16, op: CycleOp) -> u8 {
        self.cycle_manager.read(address, op)
    }

    fn write(&mut self, address: u16, value: u8, op: CycleOp) {
        self.cycle_manager.write(address, value, op);
    }

    fn imm(&mut self) -> u8 {
        let value = self.read(self.registers.program_counter, CycleOp::None);
        self.inc_program_counter();

        value
    }

    fn inc_program_counter(&mut self) {
        self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
    }

    fn push(&mut self, value: u8) {
        self.write(
            0x100 + (self.registers.stack_pointer as u16),
            value,
            CycleOp::None,
        );

        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);

        self.read(0x100 + (self.registers.stack_pointer as u16), CycleOp::None)
    }

    fn phantom_stack_read(&mut self) {
        self.phantom_read(0x100 + (self.registers.stack_pointer as u16));
    }

    fn push_16(&mut self, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.push(high);
        self.push(low);
    }

    fn pop_16(&mut self) -> u16 {
        u16::from_le_bytes([self.pop(), self.pop()])
    }

    fn address(&mut self, address_fn: AddressFn<T>) -> u16 {
        address_fn(self.cycle_manager, self.registers)
    }

    fn data(&mut self, data_fn: DataFn<T>) -> u8 {
        data_fn(self.cycle_manager, self.registers)
    }

    fn nop_read(&mut self, data_fn: DataFn<T>) {
        self.data(data_fn);
    }

    fn store(&mut self, value: u8, address_fn: AddressFn<T>) {
        let address = self.address(address_fn);

        self.write(address, value, CycleOp::CheckInterrupt);
    }

    fn read_modify_write(
        &mut self,
        unary_op: fn(&mut ProcessorFlags, u8) -> u8,
        address_fn: AddressFn<T>,
    ) {
        self.read_modify_write_return_value(unary_op, address_fn);
    }

    fn read_modify_write_with_accumulator_op(
        &mut self,
        unary_op: fn(&mut ProcessorFlags, u8) -> u8,
        accumulator_op: fn(&mut ProcessorFlags, u8, u8) -> u8,
        address_fn: AddressFn<T>,
    ) {
        let new_value = self.read_modify_write_return_value(unary_op, address_fn);

        self.registers.accumulator = accumulator_op(
            &mut self.registers.processor_flags,
            self.registers.accumulator,
            new_value,
        );
    }

    fn read_modify_write_return_value(
        &mut self,
        unary_op: fn(&mut ProcessorFlags, u8) -> u8,
        address_fn: AddressFn<T>,
    ) -> u8 {
        let address = self.address(address_fn);

        let old_value = self.read(address, CycleOp::Sync);

        self.write(address, old_value, CycleOp::Sync);

        let new_value = unary_op(&mut self.registers.processor_flags, old_value);

        self.write(address, new_value, CycleOp::Sync);

        new_value
    }

    fn accumulator_unary_op(&mut self, unary_op: fn(&mut ProcessorFlags, u8) -> u8) {
        self.phantom_program_counter_read();

        self.registers.accumulator = unary_op(
            &mut self.registers.processor_flags,
            self.registers.accumulator,
        );
    }

    fn x_index_unary_op(&mut self, unary_op: fn(&mut ProcessorFlags, u8) -> u8) {
        self.phantom_program_counter_read();

        self.registers.x_index =
            unary_op(&mut self.registers.processor_flags, self.registers.x_index);
    }

    fn y_index_unary_op(&mut self, unary_op: fn(&mut ProcessorFlags, u8) -> u8) {
        self.phantom_program_counter_read();

        self.registers.y_index =
            unary_op(&mut self.registers.processor_flags, self.registers.y_index);
    }

    fn accumulator_binary_op(
        &mut self,
        binary_op: fn(&mut ProcessorFlags, u8, u8) -> u8,
        data_fn: DataFn<T>,
    ) {
        let operand = self.data(data_fn);

        self.registers.accumulator = binary_op(
            &mut self.registers.processor_flags,
            self.registers.accumulator,
            operand,
        );
    }

    fn processor_flags_op<F: Fn(&mut ProcessorFlags)>(&mut self, callback: F) {
        self.phantom_program_counter_read();

        callback(&mut self.registers.processor_flags);
    }

    fn break_inst(
        &mut self,
        return_address_offset: u8,
        additional_processor_flags: u8,
        interrupt_vector: u16,
    ) {
        self.phantom_program_counter_read();

        let return_address = self
            .registers
            .program_counter
            .wrapping_add(return_address_offset as u16);

        self.push_16(return_address);

        self.push(u8::from(&self.registers.processor_flags) | additional_processor_flags);

        self.registers.processor_flags.interrupt_disable = true;

        self.registers.program_counter = u16::from_le_bytes([
            self.read(interrupt_vector, CycleOp::None),
            self.read(interrupt_vector + 1, CycleOp::None),
        ]);
    }

    fn jump_to_subroutine(&mut self) {
        let program_counter_low = self.imm();

        self.phantom_stack_read();

        self.push_16(self.registers.program_counter);

        let program_counter_high = self.imm();

        self.registers.program_counter =
            u16::from_le_bytes([program_counter_low, program_counter_high]);
    }

    fn jump(&mut self, address_mode: AddressFn<T>) {
        self.registers.program_counter = self.address(address_mode);
    }

    fn return_from_interrupt(&mut self) {
        self.phantom_program_counter_read();

        self.phantom_stack_read();

        self.registers.processor_flags = self.pop().into();

        self.registers.program_counter = self.pop_16();
    }

    fn return_from_subroutine(&mut self) {
        self.phantom_program_counter_read();

        self.phantom_stack_read();

        self.registers.program_counter = self.pop_16();

        self.phantom_program_counter_read();

        self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
    }

    fn pull_accumulator(&mut self) {
        self.phantom_program_counter_read();

        self.phantom_stack_read();

        self.registers.accumulator = self.pop();

        self.registers
            .processor_flags
            .update_zero_negative(self.registers.accumulator);
    }

    fn push_accumulator(&mut self) {
        self.phantom_program_counter_read();

        self.push(self.registers.accumulator);
    }

    fn pull_processor_flags(&mut self) {
        self.phantom_program_counter_read();

        self.phantom_stack_read();

        self.registers.processor_flags = self.pop().into();
    }

    fn push_processor_flags(&mut self) {
        self.phantom_program_counter_read();

        self.push(u8::from(&self.registers.processor_flags) | P_BREAK);
    }

    fn branch(&mut self, condition: bool) {
        if !condition {
            self.phantom_program_counter_read();

            self.inc_program_counter();

            return;
        }

        self.registers.program_counter = self.address(relative_address);
    }

    fn compare(&mut self, register_value: u8, data_fn: DataFn<T>) {
        let value = self.data(data_fn);

        self.registers.processor_flags.carry = register_value >= value;
        self.registers.processor_flags.zero = register_value == value;

        let diff = register_value.wrapping_sub(value);
        self.registers.processor_flags.update_negative(diff);
    }

    fn load_register(&mut self, set_register: fn(&mut Registers, u8), data_fn: DataFn<T>) {
        let value = self.data(data_fn);

        set_register(self.registers, value);

        self.registers.processor_flags.update_zero_negative(value);
    }

    fn transfer_register(&mut self, value: u8, set_register: fn(&mut Registers, u8)) {
        self.phantom_program_counter_read();

        set_register(self.registers, value);

        self.registers.processor_flags.update_zero_negative(value);
    }

    fn transfer_register_no_flags(&mut self, value: u8, set_register: fn(&mut Registers, u8)) {
        self.phantom_program_counter_read();

        set_register(self.registers, value);
    }

    fn store_high_address_and_y(&mut self, address_fn: AddressWithCarryFn<T>) {
        let (address, carried) = address_fn(self.cycle_manager, self.registers);

        let [low, high] = address.to_le_bytes();

        if carried {
            let value = self.registers.y_index & high;

            let address = u16::from_le_bytes([low, value]);

            self.write(address, value, CycleOp::CheckInterrupt);
        } else {
            let value = self.registers.y_index & high.wrapping_add(1);

            self.write(address, value, CycleOp::CheckInterrupt);
        };
    }
}

pub const NMI_VECTOR: u16 = 0xfffa;
pub const RESET_VECTOR: u16 = 0xfffc;
pub const IRQ_BRK_VECTOR: u16 = 0xfffe;
