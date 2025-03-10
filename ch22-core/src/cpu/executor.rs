use core::panic;

use crate::cycle_manager::*;

use super::registers::*;

mod binary_ops;
mod unary_ops;

use binary_ops::*;
use unary_ops::*;

pub struct Executor<'a, T>
where
    T: CycleManagerTrait + 'a,
{
    cycle_manager: &'a mut T,
    registers: &'a mut Registers,
}

impl<'a, T> Executor<'a, T>
where
    T: CycleManagerTrait + 'a,
{
    pub fn new(cycle_manager: &'a mut T, registers: &'a mut Registers) -> Self {
        Executor {
            cycle_manager,
            registers,
        }
    }

    pub fn interrupt(&mut self, nmi: bool) {
        self.phantom_program_counter_read();

        self.brk(
            self.registers.program_counter,
            u8::from(&self.registers.processor_flags),
            if nmi { NMI_VECTOR } else { IRQ_BRK_VECTOR },
        );

        self.cycle_manager.complete();
    }

    pub fn execute(&mut self, allow_untested_in_wild: bool) {
        let opcode = self.imm();

        if [0x35, 0x36, 0x41, 0x56, 0x5e, 0xe1].contains(&opcode) && !allow_untested_in_wild {
            panic!("untested opcode: {:02x}", opcode);
        }

        match opcode {
            // BRK
            0x00 => self.brk(
                self.registers.program_counter.wrapping_add(1),
                u8::from(&self.registers.processor_flags) | P_BREAK,
                IRQ_BRK_VECTOR,
            ),

            // ORA (zp,X)
            0x01 => self.accumulator_binary_op(or, AddressMode::IndexedIndirectX),

            // DOP zp
            0x04 => self.nop_read(AddressMode::ZeroPage),

            // ORA zp
            0x05 => self.accumulator_binary_op(or, AddressMode::ZeroPage),

            // ASL zp
            0x06 => self.read_modify_write(shift_left, AddressMode::ZeroPage),

            // SLO zp
            0x07 => {
                self.read_modify_write_with_accumulator_op(shift_left, or, AddressMode::ZeroPage)
            }

            // PHP
            0x08 => self.php(),

            // ORA imm
            0x09 => self.accumulator_binary_op(or, AddressMode::Immediate),

            // ASL A
            0x0a => self.accumulator_unary_op(shift_left),

            // ANC imm
            0x0b => self.accumulator_binary_op(and_negative_carry, AddressMode::Immediate),

            // ORA abs
            0x0d => self.accumulator_binary_op(or, AddressMode::Absolute),

            // ASL abs
            0x0e => self.read_modify_write(shift_left, AddressMode::Absolute),

            // BPL rel
            0x10 => self.branch(!self.registers.processor_flags.negative),

            // ORA (zp),Y
            0x11 => self.accumulator_binary_op(or, AddressMode::IndirectIndexedY),

            // ORA zp,X
            0x15 => self.accumulator_binary_op(or, AddressMode::ZeroPageX),

            // ASL zp,X
            0x16 => self.read_modify_write(shift_left, AddressMode::ZeroPageX),

            // CLC
            0x18 => self.processor_flags_op(|flags| flags.carry = false),

            // ORA abs,X
            0x1d => self.accumulator_binary_op(or, AddressMode::AbsoluteX),

            // ASL abs,X
            0x1e => self.read_modify_write(shift_left, AddressMode::AbsoluteX),

            // ORA abs,Y
            0x19 => self.accumulator_binary_op(or, AddressMode::AbsoluteY),

            // JSR abs
            0x20 => self.jsr(),

            // AND (zp,X)
            0x21 => self.accumulator_binary_op(and, AddressMode::IndexedIndirectX),

            // BIT zp
            0x24 => self.accumulator_binary_op(bit_test, AddressMode::ZeroPage),

            // AND zp
            0x25 => self.accumulator_binary_op(and, AddressMode::ZeroPage),

            // ROL zp
            0x26 => self.read_modify_write(rotate_left, AddressMode::ZeroPage),

            // PLP
            0x28 => self.plp(),

            // AND imm
            0x29 => self.accumulator_binary_op(and, AddressMode::Immediate),

            // ROL A
            0x2a => self.accumulator_unary_op(rotate_left),

            // BIT abs
            0x2c => self.accumulator_binary_op(bit_test, AddressMode::Absolute),

            // AND abs
            0x2d => self.accumulator_binary_op(and, AddressMode::Absolute),

            // ROL abs
            0x2e => self.read_modify_write(rotate_left, AddressMode::Absolute),

            // BMI rel
            0x30 => self.branch(self.registers.processor_flags.negative),

            // AND (zp),Y
            0x31 => self.accumulator_binary_op(and, AddressMode::IndirectIndexedY),

            // AND zp,X
            0x35 => self.accumulator_binary_op(and, AddressMode::ZeroPageX),

            // ROL zp,X
            0x36 => self.read_modify_write(rotate_left, AddressMode::ZeroPageX),

            // SEC
            0x38 => self.processor_flags_op(|flags| flags.carry = true),

            // AND abs,Y
            0x39 => self.accumulator_binary_op(and, AddressMode::AbsoluteY),

            // AND abs,X
            0x3d => self.accumulator_binary_op(and, AddressMode::AbsoluteX),

            // ROL abs,X
            0x3e => self.read_modify_write(rotate_left, AddressMode::AbsoluteX),

            // RTI
            0x40 => self.rti(),

            // EOR (zp,X)
            0x41 => self.accumulator_binary_op(xor, AddressMode::IndexedIndirectX),

            // EOR zp
            0x45 => self.accumulator_binary_op(xor, AddressMode::ZeroPage),

            // LSR zp
            0x46 => self.read_modify_write(shift_right, AddressMode::ZeroPage),

            // PHA
            0x48 => self.pha(),

            // EOR imm
            0x49 => self.accumulator_binary_op(xor, AddressMode::Immediate),

            // LSR A
            0x4a => self.accumulator_unary_op(shift_right),

            // ALR imm
            0x4b => self.accumulator_binary_op(and_shift_right, AddressMode::Immediate),

            // JMP abs
            0x4c => self.jmp(AddressMode::Absolute),

            // EOR abs
            0x4d => self.accumulator_binary_op(xor, AddressMode::Absolute),

            // LSR abs
            0x4e => self.read_modify_write(shift_right, AddressMode::Absolute),

            // BVC rel
            0x50 => self.branch(!self.registers.processor_flags.overflow),

            // EOR (zp),Y
            0x51 => self.accumulator_binary_op(xor, AddressMode::IndirectIndexedY),

            // EOR zp,X
            0x55 => self.accumulator_binary_op(xor, AddressMode::ZeroPageX),

            // LSR zp,X
            0x56 => self.read_modify_write(shift_right, AddressMode::ZeroPageX),

            // CLI
            0x58 => self.processor_flags_op(|flags| flags.interrupt_disable = false),

            // EOR abs,Y
            0x59 => self.accumulator_binary_op(xor, AddressMode::AbsoluteY),

            // EOR abs,X
            0x5d => self.accumulator_binary_op(xor, AddressMode::AbsoluteX),

            // LSR abs,X
            0x5e => self.read_modify_write(shift_right, AddressMode::AbsoluteX),

            // RTS
            0x60 => self.rts(),

            // ADC (zp,X)
            0x61 => self.accumulator_binary_op(add_with_carry, AddressMode::IndexedIndirectX),

            // ADC zp
            0x65 => self.accumulator_binary_op(add_with_carry, AddressMode::ZeroPage),

            // ROR zp
            0x66 => self.read_modify_write(rotate_right, AddressMode::ZeroPage),

            // PLA
            0x68 => self.pla(),

            // ADC imm
            0x69 => self.accumulator_binary_op(add_with_carry, AddressMode::Immediate),

            // ROR A
            0x6a => self.accumulator_unary_op(rotate_right),

            // JMP (abs)
            0x6c => self.jmp(AddressMode::Indirect),

            // ADC abs
            0x6d => self.accumulator_binary_op(add_with_carry, AddressMode::Absolute),

            // ROR abs
            0x6e => self.read_modify_write(rotate_right, AddressMode::Absolute),

            // BVS rel
            0x70 => self.branch(self.registers.processor_flags.overflow),

            // ADC (zp)
            0x71 => self.accumulator_binary_op(add_with_carry, AddressMode::IndirectIndexedY),

            // ADC zp,X
            0x75 => self.accumulator_binary_op(add_with_carry, AddressMode::ZeroPageX),

            // ROR zp,X
            0x76 => self.read_modify_write(rotate_right, AddressMode::ZeroPageX),

            // SEI
            0x78 => self.processor_flags_op(|flags| flags.interrupt_disable = true),

            // ADC abs,Y
            0x79 => self.accumulator_binary_op(add_with_carry, AddressMode::AbsoluteY),

            // ADC abs,X
            0x7d => self.accumulator_binary_op(add_with_carry, AddressMode::AbsoluteX),

            // ROR abs,X
            0x7e => self.read_modify_write(rotate_right, AddressMode::AbsoluteX),

            // STA (zp,X)
            0x81 => self.store(self.registers.accumulator, AddressMode::IndexedIndirectX),

            // STY zp
            0x84 => self.store(self.registers.y_index, AddressMode::ZeroPage),

            // STA zp
            0x85 => self.store(self.registers.accumulator, AddressMode::ZeroPage),

            // STX zp
            0x86 => self.store(self.registers.x_index, AddressMode::ZeroPage),

            // SAX zp
            0x87 => self.store(
                self.registers.accumulator & self.registers.x_index,
                AddressMode::ZeroPage,
            ),

            // DEY
            0x88 => self.y_index_unary_op(decrement),

            // TXA
            0x8a => self.transfer_register(self.registers.x_index, set_accumulator),

            // STY abs
            0x8c => self.store(self.registers.y_index, AddressMode::Absolute),

            // STA abs
            0x8d => self.store(self.registers.accumulator, AddressMode::Absolute),

            // STX abs
            0x8e => self.store(self.registers.x_index, AddressMode::Absolute),

            // BCC rel
            0x90 => self.branch(!self.registers.processor_flags.carry),

            // STA (zp),Y
            0x91 => self.store(self.registers.accumulator, AddressMode::IndirectIndexedY),

            // STY zp,X
            0x94 => self.store(self.registers.y_index, AddressMode::ZeroPageX),

            // STA zp,X
            0x95 => self.store(self.registers.accumulator, AddressMode::ZeroPageX),

            // STX zp,Y
            0x96 => self.store(self.registers.x_index, AddressMode::ZeroPageY),

            // STA abs,Y
            0x99 => self.store(self.registers.accumulator, AddressMode::AbsoluteY),

            // TYA
            0x98 => self.transfer_register(self.registers.y_index, set_accumulator),

            // TXS
            0x9a => self.transfer_register_no_flags(self.registers.x_index, set_stack_pointer),

            // SHY abs,X
            0x9c => self.shy(AddressMode::AbsoluteX),

            // STA abs,X
            0x9d => self.store(self.registers.accumulator, AddressMode::AbsoluteX),

            // LDY imm
            0xa0 => self.load_register(set_y_index, AddressMode::Immediate),

            // LDA (zp,X)
            0xa1 => self.load_register(set_accumulator, AddressMode::IndexedIndirectX),

            // LDX imm
            0xa2 => self.load_register(set_x_index, AddressMode::Immediate),

            // LDY zp
            0xa4 => self.load_register(set_y_index, AddressMode::ZeroPage),

            // LDA zp
            0xa5 => self.load_register(set_accumulator, AddressMode::ZeroPage),

            // LDX zp
            0xa6 => self.load_register(set_x_index, AddressMode::ZeroPage),

            // TAY
            0xa8 => self.transfer_register(self.registers.accumulator, set_y_index),

            // LDA imm
            0xa9 => self.load_register(set_accumulator, AddressMode::Immediate),

            // TXA
            0xaa => self.transfer_register(self.registers.accumulator, set_x_index),

            // LDY abs
            0xac => self.load_register(set_y_index, AddressMode::Absolute),

            // LDA abs
            0xad => self.load_register(set_accumulator, AddressMode::Absolute),

            // LDX abs
            0xae => self.load_register(set_x_index, AddressMode::Absolute),

            // BCS rel
            0xb0 => self.branch(self.registers.processor_flags.carry),

            // LDA (zp),Y
            0xb1 => self.load_register(set_accumulator, AddressMode::IndirectIndexedY),

            // LDY zp,X
            0xb4 => self.load_register(set_y_index, AddressMode::ZeroPageX),

            // LDA zp,X
            0xb5 => self.load_register(set_accumulator, AddressMode::ZeroPageX),

            // LDX zp,Y
            0xb6 => self.load_register(set_x_index, AddressMode::ZeroPageY),

            // CLV
            0xb8 => self.processor_flags_op(|flags| flags.overflow = false),

            // LDA abs,Y
            0xb9 => self.load_register(set_accumulator, AddressMode::AbsoluteY),

            // TSX
            0xba => self.transfer_register(self.registers.stack_pointer, set_x_index),

            // LDY abs,X
            0xbc => self.load_register(set_y_index, AddressMode::AbsoluteX),

            // LDA abs,X
            0xbd => self.load_register(set_accumulator, AddressMode::AbsoluteX),

            // LDX abs,Y
            0xbe => self.load_register(set_x_index, AddressMode::AbsoluteY),

            // CPY imm
            0xc0 => self.compare(self.registers.y_index, AddressMode::Immediate),

            // CMP (zp,X)
            0xc1 => self.compare(self.registers.accumulator, AddressMode::IndexedIndirectX),

            // CPY zp
            0xc4 => self.compare(self.registers.y_index, AddressMode::ZeroPage),

            // CMP zp
            0xc5 => self.compare(self.registers.accumulator, AddressMode::ZeroPage),

            // DEC zp
            0xc6 => self.read_modify_write(decrement, AddressMode::ZeroPage),

            // INY
            0xc8 => self.y_index_unary_op(increment),

            // CMP abs
            0xc9 => self.compare(self.registers.accumulator, AddressMode::Immediate),

            // DEX
            0xca => self.x_index_unary_op(decrement),

            // CPY abs
            0xcc => self.compare(self.registers.y_index, AddressMode::Absolute),

            // CMP abs
            0xcd => self.compare(self.registers.accumulator, AddressMode::Absolute),

            // DEC abs
            0xce => self.read_modify_write(decrement, AddressMode::Absolute),

            // BNE rel
            0xd0 => self.branch(!self.registers.processor_flags.zero),

            // CMP (zp),Y
            0xd1 => self.compare(self.registers.accumulator, AddressMode::IndirectIndexedY),

            // CMP zp,X
            0xd5 => self.compare(self.registers.accumulator, AddressMode::ZeroPageX),

            // DEC zp,X
            0xd6 => self.read_modify_write(decrement, AddressMode::ZeroPageX),

            // CLD
            0xd8 => self.processor_flags_op(|flags| flags.decimal_mode = false),

            // CMP abs,Y
            0xd9 => self.compare(self.registers.accumulator, AddressMode::AbsoluteY),

            // NOP abs,X
            0xdc => self.nop_read(AddressMode::AbsoluteX),

            // CMP abs,X
            0xdd => self.compare(self.registers.accumulator, AddressMode::AbsoluteX),

            // DEC abs,X
            0xde => self.read_modify_write(decrement, AddressMode::AbsoluteX),

            // CPX imm
            0xe0 => self.compare(self.registers.x_index, AddressMode::Immediate),

            // SBC (zp,X)
            0xe1 => self.accumulator_binary_op(subtract_with_carry, AddressMode::IndexedIndirectX),

            // CPX zp
            0xe4 => self.compare(self.registers.x_index, AddressMode::ZeroPage),

            // SBC zp
            0xe5 => self.accumulator_binary_op(subtract_with_carry, AddressMode::ZeroPage),

            // INC zp
            0xe6 => self.read_modify_write(increment, AddressMode::ZeroPage),

            // INX
            0xe8 => self.x_index_unary_op(increment),

            // SBC imm
            0xe9 => self.accumulator_binary_op(subtract_with_carry, AddressMode::Immediate),

            // NOP
            0xea => self.phantom_program_counter_read(),

            // CPX abs
            0xec => self.compare(self.registers.x_index, AddressMode::Absolute),

            // SBC abs
            0xed => self.accumulator_binary_op(subtract_with_carry, AddressMode::Absolute),

            // INC abs
            0xee => self.read_modify_write(increment, AddressMode::Absolute),

            // BEQ rel
            0xf0 => self.branch(self.registers.processor_flags.zero),

            // SBC (zp),Y
            0xf1 => self.accumulator_binary_op(subtract_with_carry, AddressMode::IndirectIndexedY),

            // SBC zp,X
            0xf5 => self.accumulator_binary_op(subtract_with_carry, AddressMode::ZeroPageX),

            // INC zp,X
            0xf6 => self.read_modify_write(increment, AddressMode::ZeroPageX),

            // SED
            0xf8 => self.processor_flags_op(|flags| flags.decimal_mode = true),

            // SBC abs,Y
            0xf9 => self.accumulator_binary_op(subtract_with_carry, AddressMode::AbsoluteY),

            // SBC abs,X
            0xfd => self.accumulator_binary_op(subtract_with_carry, AddressMode::AbsoluteX),

            // INC abs,X
            0xfe => self.read_modify_write(increment, AddressMode::AbsoluteX),

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

    fn address(&mut self, address_mode: AddressMode) -> u16 {
        match address_mode {
            AddressMode::ZeroPage => self.imm() as u16,
            AddressMode::ZeroPageX => {
                let base_address = self.imm();

                self.phantom_read(base_address as u16);

                base_address.wrapping_add(self.registers.x_index) as u16
            }
            AddressMode::ZeroPageY => {
                let base_address = self.imm();

                self.phantom_read(base_address as u16);

                base_address.wrapping_add(self.registers.y_index) as u16
            }
            AddressMode::Relative => {
                let rel_address = self.imm() as i8;

                self.phantom_program_counter_read();

                let (address, carry_result) =
                    address_offset_signed(self.registers.program_counter, rel_address);

                if let CarryResult::Carried { intermediate } = carry_result {
                    self.phantom_read(intermediate);
                }

                address
            }
            AddressMode::Absolute => u16::from_le_bytes([self.imm(), self.imm()]),
            AddressMode::AbsoluteX | AddressMode::AbsoluteY => {
                let offset = match address_mode {
                    AddressMode::AbsoluteX => self.registers.x_index,
                    AddressMode::AbsoluteY => self.registers.y_index,
                    _ => panic!(""),
                };

                let (address, carry_result) =
                    address_offset_unsigned(u16::from_le_bytes([self.imm(), self.imm()]), offset);

                if let CarryResult::Carried { intermediate } = carry_result {
                    self.phantom_read(intermediate);
                } else {
                    self.phantom_read(address);
                }

                address
            }
            AddressMode::Indirect => {
                let base_address = u16::from_le_bytes([self.imm(), self.imm()]);

                u16::from_le_bytes([
                    self.read(base_address, CycleOp::None),
                    self.read(next_address_same_page(base_address), CycleOp::None),
                ])
            }
            AddressMode::IndexedIndirectX => {
                let address = self.address(AddressMode::ZeroPageX);

                u16::from_le_bytes([
                    self.read(address, CycleOp::None),
                    self.read((address + 1) & 0xff, CycleOp::None),
                ])
            }
            AddressMode::IndirectIndexedY => {
                let (address, carry_result) =
                    address_offset_unsigned(self.zpg_address_value_16(), self.registers.y_index);

                if let CarryResult::Carried { intermediate } = carry_result {
                    self.phantom_read(intermediate);
                } else {
                    self.phantom_read(address);
                }

                address
            }
            _ => panic!(),
        }
    }

    fn data(&mut self, address_mode: AddressMode) -> u8 {
        match address_mode {
            AddressMode::Immediate => self.imm(),
            AddressMode::ZeroPage
            | AddressMode::ZeroPageX
            | AddressMode::ZeroPageY
            | AddressMode::Absolute
            | AddressMode::IndexedIndirectX => {
                let address = self.address(address_mode);

                self.read(address, CycleOp::CheckInterrupt)
            }
            AddressMode::AbsoluteX | AddressMode::AbsoluteY => {
                let offset = match address_mode {
                    AddressMode::AbsoluteX => self.registers.x_index,
                    AddressMode::AbsoluteY => self.registers.y_index,
                    _ => panic!(""),
                };

                let (address, carry_result) =
                    address_offset_unsigned(self.address(AddressMode::Absolute), offset);

                if let CarryResult::Carried { intermediate } = carry_result {
                    self.phantom_read(intermediate);
                }

                self.read(address, CycleOp::CheckInterrupt)
            }
            AddressMode::IndirectIndexedY => {
                let (address, carry_result) =
                    address_offset_unsigned(self.zpg_address_value_16(), self.registers.y_index);

                if let CarryResult::Carried { intermediate } = carry_result {
                    self.phantom_read(intermediate);
                }

                self.read(address, CycleOp::CheckInterrupt)
            }

            _ => panic!(),
        }
    }

    fn nop_read(&mut self, address_mode: AddressMode) {
        self.data(address_mode);
    }

    fn zpg_address_value_16(&mut self) -> u16 {
        let zpg_address = self.address(AddressMode::ZeroPage);

        u16::from_le_bytes([
            self.read(zpg_address, CycleOp::None),
            self.read((zpg_address + 1) & 0xff, CycleOp::None),
        ])
    }

    fn store(&mut self, value: u8, address_mode: AddressMode) {
        let address = self.address(address_mode);

        self.write(address, value, CycleOp::CheckInterrupt);
    }

    fn read_modify_write(
        &mut self,
        unary_op: fn(&mut ProcessorFlags, u8) -> u8,
        address_mode: AddressMode,
    ) {
        self.read_modify_write_with_return(unary_op, address_mode);
    }

    fn read_modify_write_with_accumulator_op(
        &mut self,
        unary_op: fn(&mut ProcessorFlags, u8) -> u8,
        accumulator_op: fn(&mut ProcessorFlags, u8, u8) -> u8,
        address_mode: AddressMode,
    ) {
        let new_value = self.read_modify_write_with_return(unary_op, address_mode);

        self.registers.accumulator = accumulator_op(
            &mut self.registers.processor_flags,
            self.registers.accumulator,
            new_value,
        );
    }

    fn read_modify_write_with_return(
        &mut self,
        unary_op: fn(&mut ProcessorFlags, u8) -> u8,
        address_mode: AddressMode,
    ) -> u8 {
        let address = self.address(address_mode);

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
        address_mode: AddressMode,
    ) {
        let operand = self.data(address_mode);

        self.registers.accumulator = binary_op(
            &mut self.registers.processor_flags,
            self.registers.accumulator,
            operand,
        );
    }

    fn processor_flags_op<F>(&mut self, callback: F)
    where
        F: Fn(&mut ProcessorFlags),
    {
        self.phantom_program_counter_read();

        callback(&mut self.registers.processor_flags);
    }

    fn brk(&mut self, return_address: u16, stack_p_flags: u8, interrupt_vector: u16) {
        self.phantom_program_counter_read();

        self.push_16(return_address);

        self.push(stack_p_flags);

        self.registers.processor_flags.interrupt_disable = true;

        self.registers.program_counter = u16::from_le_bytes([
            self.read(interrupt_vector, CycleOp::None),
            self.read(interrupt_vector + 1, CycleOp::None),
        ]);
    }

    fn jsr(&mut self) {
        let program_counter_low = self.imm();

        self.phantom_stack_read();

        self.push_16(self.registers.program_counter);

        let program_counter_high = self.imm();

        self.registers.program_counter =
            u16::from_le_bytes([program_counter_low, program_counter_high]);
    }

    fn jmp(&mut self, address_mode: AddressMode) {
        self.registers.program_counter = self.address(address_mode);
    }

    fn rti(&mut self) {
        self.phantom_program_counter_read();

        self.phantom_stack_read();

        let processor_flags = self.pop();

        self.registers.processor_flags = processor_flags.into();

        self.registers.program_counter = self.pop_16();
    }

    fn rts(&mut self) {
        self.phantom_program_counter_read();

        self.phantom_stack_read();

        self.registers.program_counter = self.pop_16();

        self.phantom_program_counter_read();

        self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
    }

    fn pla(&mut self) {
        self.phantom_program_counter_read();

        self.phantom_stack_read();

        self.registers.accumulator = self.pop();

        self.registers
            .processor_flags
            .update_zero_negative(self.registers.accumulator);
    }

    fn pha(&mut self) {
        self.phantom_program_counter_read();

        self.push(self.registers.accumulator);
    }

    fn php(&mut self) {
        self.phantom_program_counter_read();

        self.push(u8::from(&self.registers.processor_flags) | P_BREAK);
    }

    fn plp(&mut self) {
        self.phantom_program_counter_read();

        self.phantom_stack_read();

        self.registers.processor_flags = self.pop().into();
    }

    fn branch(&mut self, condition: bool) {
        if !condition {
            self.phantom_program_counter_read();

            self.inc_program_counter();

            return;
        }

        self.registers.program_counter = self.address(AddressMode::Relative);
    }

    fn compare(&mut self, register_value: u8, address_mode: AddressMode) {
        let value = self.data(address_mode);

        self.registers.processor_flags.carry = register_value >= value;
        self.registers.processor_flags.zero = register_value == value;

        let diff = register_value.wrapping_sub(value);
        self.registers.processor_flags.update_negative(diff);
    }

    fn load_register(&mut self, set_register: fn(&mut Registers, u8), address_mode: AddressMode) {
        let value = self.data(address_mode);

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

    fn shy(&mut self, address_mode: AddressMode) {
        let AddressMode::AbsoluteX = address_mode else {
            panic!();
        };

        let (address, carry_result) =
            address_offset_unsigned(self.address(AddressMode::Absolute), self.registers.x_index);

        if let CarryResult::Carried { intermediate } = carry_result {
            self.phantom_read(intermediate);
        } else {
            self.phantom_read(address);
        }

        let carried = if let CarryResult::Carried { .. } = carry_result {
            true
        } else {
            false
        };

        let [low, high] = address.to_le_bytes();

        let value = self.registers.y_index & if carried { high } else { high.wrapping_add(1) };

        let address = if carried {
            u16::from_le_bytes([low, value])
        } else {
            address
        };

        self.write(address, value, CycleOp::CheckInterrupt);
    }
}

fn next_address_same_page(address: u16) -> u16 {
    let [address_low, address_high] = address.to_le_bytes();

    u16::from_le_bytes([address_low.wrapping_add(1), address_high])
}

fn address_offset(base_address: u16, offset: i16) -> (u16, CarryResult) {
    let address = base_address.wrapping_add(offset as u16);

    let carried = address & 0xff00 != base_address & 0xff00;

    if carried {
        let intermediate = (base_address & 0xff00) | (address & 0x00ff);
        (address, CarryResult::Carried { intermediate })
    } else {
        (address, CarryResult::NoCarry)
    }
}

fn address_offset_unsigned(base_address: u16, offset: u8) -> (u16, CarryResult) {
    address_offset(base_address, offset as i16)
}

fn address_offset_signed(base_address: u16, offset: i8) -> (u16, CarryResult) {
    address_offset(base_address, offset as i16)
}

enum CarryResult {
    Carried { intermediate: u16 },
    NoCarry,
}

pub const NMI_VECTOR: u16 = 0xfffa;
pub const RESET_VECTOR: u16 = 0xfffc;
pub const IRQ_BRK_VECTOR: u16 = 0xfffe;

enum AddressMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirectX,
    IndirectIndexedY,
}
