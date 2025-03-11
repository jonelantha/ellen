use crate::cpu::registers::Registers;
use crate::cycle_manager::{CycleManagerTrait, CycleOp};

pub type AddressFn<T> = fn(cycle_manager: &mut T, registers: &mut Registers) -> u16;

pub fn zero_page_address<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u16 {
    read_immediate(cycle_manager, registers) as u16
}

pub fn zero_page_x_address<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u16 {
    let base_address = read_immediate(cycle_manager, registers);

    cycle_manager.phantom_read(base_address as u16);

    base_address.wrapping_add(registers.x_index) as u16
}

pub fn zero_page_y_address<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u16 {
    let base_address = read_immediate(cycle_manager, registers);

    cycle_manager.phantom_read(base_address as u16);

    base_address.wrapping_add(registers.y_index) as u16
}

pub fn relative_address<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u16 {
    let rel_address = read_immediate(cycle_manager, registers) as i8;

    phantom_program_counter_read(cycle_manager, registers);

    let (address, carry_result) = address_offset_signed(registers.program_counter, rel_address);

    if let CarryResult::Carried { intermediate } = carry_result {
        cycle_manager.phantom_read(intermediate);
    }

    address
}

pub fn absolute_address<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u16 {
    u16::from_le_bytes([
        read_immediate(cycle_manager, registers),
        read_immediate(cycle_manager, registers),
    ])
}

pub fn absolute_x_address<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u16 {
    absolute_offset_address(cycle_manager, registers, registers.x_index).0
}

pub fn absolute_y_address<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u16 {
    absolute_offset_address(cycle_manager, registers, registers.y_index).0
}

pub fn indirect_address<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u16 {
    let base_address = u16::from_le_bytes([
        read_immediate(cycle_manager, registers),
        read_immediate(cycle_manager, registers),
    ]);

    u16::from_le_bytes([
        cycle_manager.read(base_address, CycleOp::None),
        cycle_manager.read(next_address_same_page(base_address), CycleOp::None),
    ])
}

pub fn indexed_indirect_x_address<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u16 {
    let address = zero_page_x_address(cycle_manager, registers);

    u16::from_le_bytes([
        cycle_manager.read(address, CycleOp::None),
        cycle_manager.read((address + 1) & 0xff, CycleOp::None),
    ])
}

pub fn indirect_indexed_y_address<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u16 {
    let (address, carry_result) = address_offset_unsigned(
        zpg_address_value_16(cycle_manager, registers),
        registers.y_index,
    );

    if let CarryResult::Carried { intermediate } = carry_result {
        cycle_manager.phantom_read(intermediate);
    } else {
        cycle_manager.phantom_read(address);
    }

    address
}

///

pub type AddressWithCarryFn<T> =
    fn(cycle_manager: &mut T, registers: &mut Registers) -> (u16, bool);

pub fn absolute_x_address_with_carry<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> (u16, bool) {
    absolute_offset_address(cycle_manager, registers, registers.x_index)
}

fn absolute_offset_address<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
    offset: u8,
) -> (u16, bool) {
    let (address, carry_result) = address_offset_unsigned(
        u16::from_le_bytes([
            read_immediate(cycle_manager, registers),
            read_immediate(cycle_manager, registers),
        ]),
        offset,
    );

    if let CarryResult::Carried { intermediate } = carry_result {
        cycle_manager.phantom_read(intermediate);

        (address, true)
    } else {
        cycle_manager.phantom_read(address);

        (address, false)
    }
}

///

pub type DataFn<T> = fn(cycle_manager: &mut T, registers: &mut Registers) -> u8;

pub fn immediate_data<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u8 {
    read_immediate(cycle_manager, registers)
}

pub fn zero_page_data<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u8 {
    let address = zero_page_address(cycle_manager, registers);

    cycle_manager.read(address, CycleOp::CheckInterrupt)
}

pub fn zero_page_x_data<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u8 {
    let address = zero_page_x_address(cycle_manager, registers);

    cycle_manager.read(address, CycleOp::CheckInterrupt)
}

pub fn zero_page_y_data<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u8 {
    let address = zero_page_y_address(cycle_manager, registers);

    cycle_manager.read(address, CycleOp::CheckInterrupt)
}

pub fn absolute_data<T: CycleManagerTrait>(cycle_manager: &mut T, registers: &mut Registers) -> u8 {
    let address = absolute_address(cycle_manager, registers);

    cycle_manager.read(address, CycleOp::CheckInterrupt)
}

pub fn absolute_x_data<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u8 {
    absolute_offset_data(cycle_manager, registers, registers.x_index)
}

pub fn absolute_y_data<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u8 {
    absolute_offset_data(cycle_manager, registers, registers.y_index)
}

fn absolute_offset_data<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
    offset: u8,
) -> u8 {
    let (address, carry_result) = address_offset_unsigned(
        u16::from_le_bytes([
            read_immediate(cycle_manager, registers),
            read_immediate(cycle_manager, registers),
        ]),
        offset,
    );

    if let CarryResult::Carried { intermediate } = carry_result {
        cycle_manager.phantom_read(intermediate);
    }

    cycle_manager.read(address, CycleOp::CheckInterrupt)
}

pub fn indexed_indirect_x_data<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u8 {
    let address = indexed_indirect_x_address(cycle_manager, registers);

    cycle_manager.read(address, CycleOp::CheckInterrupt)
}

pub fn indirect_indexed_y_data<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u8 {
    let (address, carry_result) = address_offset_unsigned(
        zpg_address_value_16(cycle_manager, registers),
        registers.y_index,
    );

    if let CarryResult::Carried { intermediate } = carry_result {
        cycle_manager.phantom_read(intermediate);
    }

    cycle_manager.read(address, CycleOp::CheckInterrupt)
}

///

fn zpg_address_value_16<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) -> u16 {
    let zpg_address = zero_page_address(cycle_manager, registers);

    u16::from_le_bytes([
        cycle_manager.read(zpg_address, CycleOp::None),
        cycle_manager.read((zpg_address + 1) & 0xff, CycleOp::None),
    ])
}

fn phantom_program_counter_read<T: CycleManagerTrait>(
    cycle_manager: &mut T,
    registers: &mut Registers,
) {
    cycle_manager.phantom_read(registers.program_counter);
}

fn read_immediate<T: CycleManagerTrait>(cycle_manager: &mut T, registers: &mut Registers) -> u8 {
    let value = cycle_manager.read(registers.program_counter, CycleOp::None);

    registers.program_counter = registers.program_counter.wrapping_add(1);

    value
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
