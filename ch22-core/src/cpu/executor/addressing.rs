use crate::cycle_manager::{CycleManagerTrait, CycleOp};

use AddressMode::*;

use super::immediate_access::*;

pub enum AddressMode {
    Immediate,
    ZeroPage,
    ZeroPageIndexed(u8),
    Absolute,
    AbsoluteIndexed(u8),
    Indirect,
    IndexedIndirect(u8),
    IndirectIndexed(u8),
    Relative,
}

impl AddressMode {
    pub fn address<T: CycleManagerTrait>(
        &self,
        cycle_manager: &mut T,
        program_counter: &mut u16,
    ) -> u16 {
        match self {
            Immediate => panic!(),

            ZeroPage => read_immediate(cycle_manager, program_counter) as u16,

            ZeroPageIndexed(index) => {
                let base_address = read_immediate(cycle_manager, program_counter);

                cycle_manager.phantom_read(base_address as u16);

                base_address.wrapping_add(*index) as u16
            }

            Absolute => read_immediate_16(cycle_manager, program_counter),

            AbsoluteIndexed(_) => self.address_with_carry(cycle_manager, program_counter).0,

            Indirect => {
                let base_address = read_immediate_16(cycle_manager, program_counter);

                read_16(cycle_manager, base_address, CycleOp::None)
            }

            IndexedIndirect(index) => {
                let address = ZeroPageIndexed(*index).address(cycle_manager, program_counter);

                read_16(cycle_manager, address, CycleOp::None)
            }

            IndirectIndexed(index) => {
                let zero_page_address = ZeroPage.address(cycle_manager, program_counter);

                let base_address = read_16(cycle_manager, zero_page_address, CycleOp::None);

                let (address, carry_result) = address_offset_unsigned(base_address, *index);

                if let CarryResult::Carried { intermediate } = carry_result {
                    cycle_manager.phantom_read(intermediate);
                } else {
                    cycle_manager.phantom_read(address);
                }

                address
            }

            Relative => {
                let rel_address = read_immediate(cycle_manager, program_counter) as i8;

                cycle_manager.phantom_read(*program_counter);

                let (address, carry_result) = address_offset_signed(*program_counter, rel_address);

                if let CarryResult::Carried { intermediate } = carry_result {
                    cycle_manager.phantom_read(intermediate);
                }

                address
            }
        }
    }

    pub fn address_with_carry<T: CycleManagerTrait>(
        &self,
        cycle_manager: &mut T,
        program_counter: &mut u16,
    ) -> (u16, bool) {
        match self {
            AbsoluteIndexed(index) => {
                let base_address = read_immediate_16(cycle_manager, program_counter);

                let (address, carry_result) = address_offset_unsigned(base_address, *index);

                if let CarryResult::Carried { intermediate } = carry_result {
                    cycle_manager.phantom_read(intermediate);

                    (address, true)
                } else {
                    cycle_manager.phantom_read(address);

                    (address, false)
                }
            }

            _ => panic!(),
        }
    }

    pub fn data<T: CycleManagerTrait>(
        &self,
        cycle_manager: &mut T,
        program_counter: &mut u16,
    ) -> u8 {
        match self {
            Immediate => read_immediate(cycle_manager, program_counter),

            ZeroPage | ZeroPageIndexed(_) | Absolute | IndexedIndirect(_) | Indirect | Relative => {
                let address = self.address(cycle_manager, program_counter);

                cycle_manager.read(address, CycleOp::CheckInterrupt)
            }

            AbsoluteIndexed(index) => {
                let base_address = read_immediate_16(cycle_manager, program_counter);

                let (address, carry_result) = address_offset_unsigned(base_address, *index);

                if let CarryResult::Carried { intermediate } = carry_result {
                    cycle_manager.phantom_read(intermediate);
                }

                cycle_manager.read(address, CycleOp::CheckInterrupt)
            }

            IndirectIndexed(index) => {
                let zero_page_address = ZeroPage.address(cycle_manager, program_counter);

                let base_address = read_16(cycle_manager, zero_page_address, CycleOp::None);

                let (address, carry_result) = address_offset_unsigned(base_address, *index);

                if let CarryResult::Carried { intermediate } = carry_result {
                    cycle_manager.phantom_read(intermediate);
                }

                cycle_manager.read(address, CycleOp::CheckInterrupt)
            }
        }
    }
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

pub fn read_16<T: CycleManagerTrait>(cycle_manager: &mut T, address: u16, op: CycleOp) -> u16 {
    u16::from_le_bytes([
        cycle_manager.read(address, op),
        cycle_manager.read(next_address_same_page(address), op),
    ])
}

fn next_address_same_page(address: u16) -> u16 {
    let [address_low, address_high] = address.to_le_bytes();

    u16::from_le_bytes([address_low.wrapping_add(1), address_high])
}
