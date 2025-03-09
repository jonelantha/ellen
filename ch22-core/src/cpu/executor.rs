use core::panic;

use crate::cycle_manager::*;

use super::registers::*;

use DataMode::*;

use RegisterType::*;

mod accumulator_ops;
mod byte_ops;

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
        self.phantom_pc_read();

        self.brk(
            self.registers.pc,
            u8::from(&self.registers.p),
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
                self.registers.pc.wrapping_add(1),
                u8::from(&self.registers.p) | P_BREAK_FLAG,
                IRQ_BRK_VECTOR,
            ),

            // ORA (zp,X)
            0x01 => self.register_op(accumulator_ops::or, IndexedIndirectX),

            // DOP zp
            0x04 => self.data_no_return(ZeroPage),

            // ORA zp
            0x05 => self.register_op(accumulator_ops::or, ZeroPage),

            // ASL zp
            0x06 => self.read_modify_write(byte_ops::shift_left, ZeroPage),

            // SLO zp
            0x07 => self.read_modify_write_accumulator(
                byte_ops::shift_left,
                accumulator_ops::or,
                ZeroPage,
            ),

            // PHP
            0x08 => self.php(),

            // ORA imm
            0x09 => self.register_op(accumulator_ops::or, Immediate),

            // ASL A
            0x0a => self.read_modify_write(byte_ops::shift_left, Register(Accumulator)),

            // ANC imm
            0x0b => self.register_op(accumulator_ops::and_negative_carry, Immediate),

            // ORA abs
            0x0d => self.register_op(accumulator_ops::or, Absolute),

            // ASL abs
            0x0e => self.read_modify_write(byte_ops::shift_left, Absolute),

            // BPL rel
            0x10 => self.branch(!self.registers.p.negative),

            // ORA (zp),Y
            0x11 => self.register_op(accumulator_ops::or, IndirectIndexedY),

            // ORA zp,X
            0x15 => self.register_op(accumulator_ops::or, ZeroPageX),

            // ASL zp,X
            0x16 => self.read_modify_write(byte_ops::shift_left, ZeroPageX),

            // CLC
            0x18 => self.flag_op(|p| p.carry = false),

            // ORA abs,X
            0x1d => self.register_op(accumulator_ops::or, AbsoluteX),

            // ASL abs,X
            0x1e => self.read_modify_write(byte_ops::shift_left, AbsoluteX),

            // ORA abs,Y
            0x19 => self.register_op(accumulator_ops::or, AbsoluteY),

            // JSR abs
            0x20 => self.jsr(),

            // AND (zp,X)
            0x21 => self.register_op(accumulator_ops::and, IndexedIndirectX),

            // BIT zp
            0x24 => self.register_op(accumulator_ops::bit_test, ZeroPage),

            // AND zp
            0x25 => self.register_op(accumulator_ops::and, ZeroPage),

            // ROL zp
            0x26 => self.read_modify_write(byte_ops::rotate_left, ZeroPage),

            // PLP
            0x28 => self.plp(),

            // AND imm
            0x29 => self.register_op(accumulator_ops::and, Immediate),

            // ROL A
            0x2a => self.read_modify_write(byte_ops::rotate_left, Register(Accumulator)),

            // BIT abs
            0x2c => self.register_op(accumulator_ops::bit_test, Absolute),

            // AND abs
            0x2d => self.register_op(accumulator_ops::and, Absolute),

            // ROL abs
            0x2e => self.read_modify_write(byte_ops::rotate_left, Absolute),

            // BMI rel
            0x30 => self.branch(self.registers.p.negative),

            // AND (zp),Y
            0x31 => self.register_op(accumulator_ops::and, IndirectIndexedY),

            // AND zp,X
            0x35 => self.register_op(accumulator_ops::and, ZeroPageX),

            // ROL zp,X
            0x36 => self.read_modify_write(byte_ops::rotate_left, ZeroPageX),

            // SEC
            0x38 => self.flag_op(|p| p.carry = true),

            // AND abs,Y
            0x39 => self.register_op(accumulator_ops::and, AbsoluteY),

            // AND abs,X
            0x3d => self.register_op(accumulator_ops::and, AbsoluteX),

            // ROL abs,X
            0x3e => self.read_modify_write(byte_ops::rotate_left, AbsoluteX),

            // RTI
            0x40 => self.rti(),

            // EOR (zp,X)
            0x41 => self.register_op(accumulator_ops::xor, IndexedIndirectX),

            // EOR zp
            0x45 => self.register_op(accumulator_ops::xor, ZeroPage),

            // LSR zp
            0x46 => self.read_modify_write(byte_ops::shift_right, ZeroPage),

            // PHA
            0x48 => self.pha(),

            // EOR imm
            0x49 => self.register_op(accumulator_ops::xor, Immediate),

            // LSR A
            0x4a => self.read_modify_write(byte_ops::shift_right, Register(Accumulator)),

            // ALR imm
            0x4b => self.register_op(accumulator_ops::and_shift_right, Immediate),

            // JMP abs
            0x4c => self.jmp(Absolute),

            // EOR abs
            0x4d => self.register_op(accumulator_ops::xor, Absolute),

            // LSR abs
            0x4e => self.read_modify_write(byte_ops::shift_right, Absolute),

            // BVC rel
            0x50 => self.branch(!self.registers.p.overflow),

            // EOR (zp),Y
            0x51 => self.register_op(accumulator_ops::xor, IndirectIndexedY),

            // EOR zp,X
            0x55 => self.register_op(accumulator_ops::xor, ZeroPageX),

            // LSR zp,X
            0x56 => self.read_modify_write(byte_ops::shift_right, ZeroPageX),

            // CLI
            0x58 => self.flag_op(|p| p.interrupt_disable = false),

            // EOR abs,Y
            0x59 => self.register_op(accumulator_ops::xor, AbsoluteY),

            // EOR abs,X
            0x5d => self.register_op(accumulator_ops::xor, AbsoluteX),

            // LSR abs,X
            0x5e => self.read_modify_write(byte_ops::shift_right, AbsoluteX),

            // RTS
            0x60 => self.rts(),

            // ADC (zp,X)
            0x61 => self.register_op(accumulator_ops::add_with_carry, IndexedIndirectX),

            // ADC zp
            0x65 => self.register_op(accumulator_ops::add_with_carry, ZeroPage),

            // ROR zp
            0x66 => self.read_modify_write(byte_ops::rotate_right, ZeroPage),

            // PLA
            0x68 => self.pla(),

            // ADC imm
            0x69 => self.register_op(accumulator_ops::add_with_carry, Immediate),

            // ROR A
            0x6a => self.read_modify_write(byte_ops::rotate_right, Register(Accumulator)),

            // JMP (abs)
            0x6c => self.jmp(Indirect),

            // ADC abs
            0x6d => self.register_op(accumulator_ops::add_with_carry, Absolute),

            // ROR abs
            0x6e => self.read_modify_write(byte_ops::rotate_right, Absolute),

            // BVS rel
            0x70 => self.branch(self.registers.p.overflow),

            // ADC (zp)
            0x71 => self.register_op(accumulator_ops::add_with_carry, IndirectIndexedY),

            // ADC zp,X
            0x75 => self.register_op(accumulator_ops::add_with_carry, ZeroPageX),

            // ROR zp,X
            0x76 => self.read_modify_write(byte_ops::rotate_right, ZeroPageX),

            // SEI
            0x78 => self.flag_op(|p| p.interrupt_disable = true),

            // ADC abs,Y
            0x79 => self.register_op(accumulator_ops::add_with_carry, AbsoluteY),

            // ADC abs,X
            0x7d => self.register_op(accumulator_ops::add_with_carry, AbsoluteX),

            // ROR abs,X
            0x7e => self.read_modify_write(byte_ops::rotate_right, AbsoluteX),

            // STA (zp,X)
            0x81 => self.store(IndexedIndirectX, self.registers.a),

            // STY zp
            0x84 => self.store(ZeroPage, self.registers.y),

            // STA zp
            0x85 => self.store(ZeroPage, self.registers.a),

            // STX zp
            0x86 => self.store(ZeroPage, self.registers.x),

            // SAX zp
            0x87 => self.store(ZeroPage, self.registers.a & self.registers.x),

            // DEY
            0x88 => self.read_modify_write(byte_ops::decrement, Register(Y)),

            // TXA
            0x8a => self.load_register(Accumulator, Register(X)),

            // STY abs
            0x8c => self.store(Absolute, self.registers.y),

            // STA abs
            0x8d => self.store(Absolute, self.registers.a),

            // STX abs
            0x8e => self.store(Absolute, self.registers.x),

            // BCC rel
            0x90 => self.branch(!self.registers.p.carry),

            // STA (zp),Y
            0x91 => self.store(IndirectIndexedY, self.registers.a),

            // STY zp,X
            0x94 => self.store(ZeroPageX, self.registers.y),

            // STA zp,X
            0x95 => self.store(ZeroPageX, self.registers.a),

            // STX zp,Y
            0x96 => self.store(ZeroPageY, self.registers.x),

            // STA abs,Y
            0x99 => self.store(AbsoluteY, self.registers.a),

            // TYA
            0x98 => self.load_register(Accumulator, Register(Y)),

            // TXS
            0x9a => self.register_inst(|registers| registers.s = registers.x),

            // SHY abs,X
            0x9c => self.shy(AbsoluteX),

            // STA abs,X
            0x9d => self.store(AbsoluteX, self.registers.a),

            // LDY imm
            0xa0 => self.load_register(Y, Immediate),

            // LDA (zp,X)
            0xa1 => self.load_register(Accumulator, IndexedIndirectX),

            // LDX imm
            0xa2 => self.load_register(X, Immediate),

            // LDY zp
            0xa4 => self.load_register(Y, ZeroPage),

            // LDA zp
            0xa5 => self.load_register(Accumulator, ZeroPage),

            // LDX zp
            0xa6 => self.load_register(X, ZeroPage),

            // TAY
            0xa8 => self.load_register(Y, Register(Accumulator)),

            // LDA imm
            0xa9 => self.load_register(Accumulator, Immediate),

            // TXA
            0xaa => self.load_register(X, Register(Accumulator)),

            // LDY abs
            0xac => self.load_register(Y, Absolute),

            // LDA abs
            0xad => self.load_register(Accumulator, Absolute),

            // LDX abs
            0xae => self.load_register(X, Absolute),

            // BCS rel
            0xb0 => self.branch(self.registers.p.carry),

            // LDA (zp),Y
            0xb1 => self.load_register(Accumulator, IndirectIndexedY),

            // LDY zp,X
            0xb4 => self.load_register(Y, ZeroPageX),

            // LDA zp,X
            0xb5 => self.load_register(Accumulator, ZeroPageX),

            // LDX zp,Y
            0xb6 => self.load_register(X, ZeroPageY),

            // CLV
            0xb8 => self.flag_op(|p| p.overflow = false),

            // LDA abs,Y
            0xb9 => self.load_register(Accumulator, AbsoluteY),

            // TSX
            0xba => self.load_register(X, Register(Stack)),

            // LDY abs,X
            0xbc => self.load_register(Y, AbsoluteX),

            // LDA abs,X
            0xbd => self.load_register(Accumulator, AbsoluteX),

            // LDX abs,Y
            0xbe => self.load_register(X, AbsoluteY),

            // CPY imm
            0xc0 => self.compare(Immediate, self.registers.y),

            // CMP (zp,X)
            0xc1 => self.compare(IndexedIndirectX, self.registers.a),

            // CPY zp
            0xc4 => self.compare(ZeroPage, self.registers.y),

            // CMP zp
            0xc5 => self.compare(ZeroPage, self.registers.a),

            // DEC zp
            0xc6 => self.read_modify_write(byte_ops::decrement, ZeroPage),

            // INY
            0xc8 => self.read_modify_write(byte_ops::increment, Register(Y)),

            // CMP abs
            0xc9 => self.compare(Immediate, self.registers.a),

            // DEX
            0xca => self.read_modify_write(byte_ops::decrement, Register(X)),

            // CPY abs
            0xcc => self.compare(Absolute, self.registers.y),

            // CMP abs
            0xcd => self.compare(Absolute, self.registers.a),

            // DEC abs
            0xce => self.read_modify_write(byte_ops::decrement, Absolute),

            // BNE rel
            0xd0 => self.branch(!self.registers.p.zero),

            // CMP (zp),Y
            0xd1 => self.compare(IndirectIndexedY, self.registers.a),

            // CMP zp,X
            0xd5 => self.compare(ZeroPageX, self.registers.a),

            // DEC zp,X
            0xd6 => self.read_modify_write(byte_ops::decrement, ZeroPageX),

            // CLD
            0xd8 => self.flag_op(|p| p.decimal_mode = false),

            // CMP abs,Y
            0xd9 => self.compare(AbsoluteY, self.registers.a),

            // NOP abs,X
            0xdc => self.data_no_return(AbsoluteX),

            // CMP abs,X
            0xdd => self.compare(AbsoluteX, self.registers.a),

            // DEC abs,X
            0xde => self.read_modify_write(byte_ops::decrement, AbsoluteX),

            // CPX imm
            0xe0 => self.compare(Immediate, self.registers.x),

            // SBC (zp,X)
            0xe1 => self.register_op(accumulator_ops::subtract_with_carry, IndexedIndirectX),

            // CPX zp
            0xe4 => self.compare(ZeroPage, self.registers.x),

            // SBC zp
            0xe5 => self.register_op(accumulator_ops::subtract_with_carry, ZeroPage),

            // INC zp
            0xe6 => self.read_modify_write(byte_ops::increment, ZeroPage),

            // INX
            0xe8 => self.read_modify_write(byte_ops::increment, Register(X)),

            // SBC imm
            0xe9 => self.register_op(accumulator_ops::subtract_with_carry, Immediate),

            // NOP
            0xea => self.phantom_pc_read(),

            // CPX abs
            0xec => self.compare(Absolute, self.registers.x),

            // SBC abs
            0xed => self.register_op(accumulator_ops::subtract_with_carry, Absolute),

            // INC abs
            0xee => self.read_modify_write(byte_ops::increment, Absolute),

            // BEQ rel
            0xf0 => self.branch(self.registers.p.zero),

            // SBC (zp),Y
            0xf1 => self.register_op(accumulator_ops::subtract_with_carry, IndirectIndexedY),

            // SBC zp,X
            0xf5 => self.register_op(accumulator_ops::subtract_with_carry, ZeroPageX),

            // INC zp,X
            0xf6 => self.read_modify_write(byte_ops::increment, ZeroPageX),

            // SED
            0xf8 => self.flag_op(|p| p.decimal_mode = true),

            // SBC abs,Y
            0xf9 => self.register_op(accumulator_ops::subtract_with_carry, AbsoluteY),

            // SBC abs,X
            0xfd => self.register_op(accumulator_ops::subtract_with_carry, AbsoluteX),

            // INC abs,X
            0xfe => self.read_modify_write(byte_ops::increment, AbsoluteX),

            _ => panic!("Unimplemented opcode: {:#04x}", opcode),
        }

        self.cycle_manager.complete();
    }

    fn phantom_read(&mut self, address: u16) {
        self.cycle_manager.phantom_read(address);
    }

    fn phantom_pc_read(&mut self) {
        self.phantom_read(self.registers.pc);
    }

    fn read(&mut self, address: u16, op: CycleOp) -> u8 {
        self.cycle_manager.read(address, op)
    }

    fn write(&mut self, address: u16, value: u8, op: CycleOp) {
        self.cycle_manager.write(address, value, op);
    }

    fn imm(&mut self) -> u8 {
        let value = self.read(self.registers.pc, CycleOp::None);
        self.inc_pc();

        value
    }

    fn inc_pc(&mut self) {
        self.registers.pc = self.registers.pc.wrapping_add(1);
    }

    fn push(&mut self, val: u8) {
        self.write(0x100 + (self.registers.s as u16), val, CycleOp::None);

        self.registers.s = self.registers.s.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.registers.s = self.registers.s.wrapping_add(1);

        self.read(0x100 + (self.registers.s as u16), CycleOp::None)
    }

    fn phantom_stack_read(&mut self) {
        self.phantom_read(0x100 + (self.registers.s as u16));
    }

    fn push_16(&mut self, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.push(high);
        self.push(low);
    }

    fn pop_16(&mut self) -> u16 {
        u16::from_le_bytes([self.pop(), self.pop()])
    }

    fn address(&mut self, address_mode: DataMode) -> u16 {
        match address_mode {
            ZeroPage => self.imm() as u16,
            ZeroPageX => {
                let base_address = self.imm();

                self.phantom_read(base_address as u16);

                base_address.wrapping_add(self.registers.x) as u16
            }
            ZeroPageY => {
                let base_address = self.imm();

                self.phantom_read(base_address as u16);

                base_address.wrapping_add(self.registers.y) as u16
            }
            Relative => {
                let rel_address = self.imm() as i8;

                self.phantom_pc_read();

                let (address, carry_result) = address_offset_signed(self.registers.pc, rel_address);

                if let CarryResult::Carried { intermediate } = carry_result {
                    self.phantom_read(intermediate);
                }

                address
            }
            Absolute => u16::from_le_bytes([self.imm(), self.imm()]),
            AbsoluteX | AbsoluteY => {
                let offset = match address_mode {
                    AbsoluteX => self.registers.x,
                    AbsoluteY => self.registers.y,
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
            Indirect => {
                let base_address = u16::from_le_bytes([self.imm(), self.imm()]);

                u16::from_le_bytes([
                    self.read(base_address, CycleOp::None),
                    self.read(next_address_same_page(base_address), CycleOp::None),
                ])
            }
            IndexedIndirectX => {
                let address = self.address(ZeroPageX);

                u16::from_le_bytes([
                    self.read(address, CycleOp::None),
                    self.read((address + 1) & 0xff, CycleOp::None),
                ])
            }
            IndirectIndexedY => {
                let (address, carry_result) =
                    address_offset_unsigned(self.zpg_address_value_16(), self.registers.y);

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

    fn data(&mut self, address_mode: DataMode) -> u8 {
        match address_mode {
            Register(register_type) => {
                self.phantom_pc_read();

                self.registers.get(register_type)
            }
            Immediate => self.imm(),
            ZeroPage | ZeroPageX | ZeroPageY | Absolute | IndexedIndirectX => {
                let address = self.address(address_mode);

                self.read(address, CycleOp::CheckInterrupt)
            }
            AbsoluteX | AbsoluteY => {
                let offset = match address_mode {
                    AbsoluteX => self.registers.x,
                    AbsoluteY => self.registers.y,
                    _ => panic!(""),
                };

                let (address, carry_result) =
                    address_offset_unsigned(self.address(Absolute), offset);

                if let CarryResult::Carried { intermediate } = carry_result {
                    self.phantom_read(intermediate);
                }

                self.read(address, CycleOp::CheckInterrupt)
            }
            IndirectIndexedY => {
                let (address, carry_result) =
                    address_offset_unsigned(self.zpg_address_value_16(), self.registers.y);

                if let CarryResult::Carried { intermediate } = carry_result {
                    self.phantom_read(intermediate);
                }

                self.read(address, CycleOp::CheckInterrupt)
            }

            _ => panic!(),
        }
    }

    fn data_no_return(&mut self, address_mode: DataMode) {
        self.data(address_mode);
    }

    fn zpg_address_value_16(&mut self) -> u16 {
        let zpg_address = self.address(ZeroPage);

        u16::from_le_bytes([
            self.read(zpg_address, CycleOp::None),
            self.read((zpg_address + 1) & 0xff, CycleOp::None),
        ])
    }

    fn store(&mut self, address_mode: DataMode, value: u8) {
        let address = self.address(address_mode);

        self.write(address, value, CycleOp::CheckInterrupt);
    }

    fn read_modify_write(&mut self, op: fn(&mut StatusRegister, u8) -> u8, address_mode: DataMode) {
        if let Register(register_type) = address_mode {
            let value = self.data(address_mode);

            let new_value = op(&mut self.registers.p, value);

            self.registers.set(register_type, new_value);
        } else {
            let address = self.address(address_mode);

            let old_value = self.read(address, CycleOp::Sync);

            self.write(address, old_value, CycleOp::Sync);

            let new_value = op(&mut self.registers.p, old_value);

            self.write(address, new_value, CycleOp::Sync);
        }
    }

    fn read_modify_write_accumulator(
        &mut self,
        op: fn(&mut StatusRegister, u8) -> u8,
        accumulator_op: fn(&mut Registers, u8),
        address_mode: DataMode,
    ) {
        if let Register(register_type) = address_mode {
            let value = self.data(address_mode);

            let new_value = op(&mut self.registers.p, value);

            self.registers.set(register_type, new_value);

            accumulator_op(&mut self.registers, new_value);
        } else {
            let address = self.address(address_mode);

            let old_value = self.read(address, CycleOp::Sync);

            self.write(address, old_value, CycleOp::Sync);

            let new_value = op(&mut self.registers.p, old_value);

            self.write(address, new_value, CycleOp::Sync);

            accumulator_op(&mut self.registers, new_value);
        }
    }

    fn flag_op<F>(&mut self, callback: F)
    where
        F: Fn(&mut StatusRegister),
    {
        self.phantom_pc_read();

        callback(&mut self.registers.p);
    }

    fn register_inst<F>(&mut self, callback: F)
    where
        F: Fn(&mut Registers),
    {
        self.phantom_pc_read();

        callback(self.registers);
    }

    fn brk(&mut self, return_address: u16, stack_p_flags: u8, interrupt_vector: u16) {
        self.phantom_pc_read();

        self.push_16(return_address);

        self.push(stack_p_flags);

        self.registers.p.interrupt_disable = true;

        self.registers.pc = u16::from_le_bytes([
            self.read(interrupt_vector, CycleOp::None),
            self.read(interrupt_vector + 1, CycleOp::None),
        ]);
    }

    fn jsr(&mut self) {
        let pc_low = self.imm();

        self.phantom_stack_read();

        self.push_16(self.registers.pc);

        let pc_high = self.imm();

        self.registers.pc = u16::from_le_bytes([pc_low, pc_high]);
    }

    fn jmp(&mut self, address_mode: DataMode) {
        self.registers.pc = self.address(address_mode);
    }

    fn rti(&mut self) {
        self.phantom_pc_read();

        self.phantom_stack_read();

        let p = self.pop();

        self.registers.p = p.into();

        self.registers.pc = self.pop_16();
    }

    fn rts(&mut self) {
        self.phantom_pc_read();

        self.phantom_stack_read();

        self.registers.pc = self.pop_16();

        self.phantom_pc_read();

        self.registers.pc = self.registers.pc.wrapping_add(1);
    }

    fn pla(&mut self) {
        self.phantom_pc_read();

        self.phantom_stack_read();

        let a = self.pop();

        self.registers.set_with_flags(Accumulator, a);
    }

    fn pha(&mut self) {
        self.phantom_pc_read();

        self.push(self.registers.a);
    }

    fn php(&mut self) {
        self.phantom_pc_read();

        self.push(u8::from(&self.registers.p) | P_BREAK_FLAG);
    }

    fn plp(&mut self) {
        self.phantom_pc_read();

        self.phantom_stack_read();

        let value = self.pop();

        self.registers.p = value.into();
    }

    fn branch(&mut self, condition: bool) {
        if !condition {
            self.phantom_pc_read();

            self.inc_pc();

            return;
        }

        self.registers.pc = self.address(Relative);
    }

    fn compare(&mut self, address_mode: DataMode, register_value: u8) {
        let value = self.data(address_mode);

        self.registers.p.carry = register_value >= value;
        self.registers.p.zero = register_value == value;

        let diff = register_value.wrapping_sub(value);
        self.registers.p.update_negative(diff);
    }

    fn load_register(&mut self, register_type: RegisterType, address_mode: DataMode) {
        let value = self.data(address_mode);

        self.registers.set_with_flags(register_type, value);
    }

    fn register_op(&mut self, op: fn(&mut Registers, u8), address_mode: DataMode) {
        let operand = self.data(address_mode);

        op(self.registers, operand);
    }

    fn shy(&mut self, address_mode: DataMode) {
        let AbsoluteX = address_mode else {
            panic!();
        };

        let (address, carry_result) =
            address_offset_unsigned(self.address(Absolute), self.registers.x);

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

        let value = self.registers.y & if carried { high } else { high.wrapping_add(1) };

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

enum DataMode {
    Register(RegisterType),
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
