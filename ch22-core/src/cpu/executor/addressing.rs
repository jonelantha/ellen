use super::memory_util::*;
use crate::bus::*;
use crate::word::*;

use AddressMode::*;

pub fn address<B: Bus>(
    bus: &mut B,
    program_counter: &mut Word,
    address_mode: &AddressMode,
) -> Word {
    match address_mode {
        Immediate => panic!(),

        ZeroPage => Word::zero_page(immediate_fetch(bus, program_counter)),

        ZeroPageIndexed(index) => {
            let base_address = Word::zero_page(immediate_fetch(bus, program_counter));

            bus.phantom_read(base_address);

            base_address.same_page_add(*index)
        }

        Absolute => immediate_fetch_word(bus, program_counter),

        AbsoluteIndexed(index) => {
            let base_address = immediate_fetch_word(bus, program_counter);

            let (address, offset_result) = base_address.paged_add(*index);

            match offset_result {
                OffsetResult::CrossedPage(intermediate) => bus.phantom_read(intermediate),

                OffsetResult::SamePage => bus.phantom_read(address),
            }

            address
        }

        Indirect => {
            let base_address = immediate_fetch_word(bus, program_counter);

            read_word(bus, base_address, CycleOp::None)
        }

        IndexedIndirect(index) => {
            let address = address(bus, program_counter, &ZeroPageIndexed(*index));

            read_word(bus, address, CycleOp::None)
        }

        IndirectIndexed(index) => {
            let zero_page_address = address(bus, program_counter, &ZeroPage);

            let base_address = read_word(bus, zero_page_address, CycleOp::None);

            let (address, offset_result) = base_address.paged_add(*index);

            match offset_result {
                OffsetResult::CrossedPage(intermediate) => bus.phantom_read(intermediate),

                OffsetResult::SamePage => bus.phantom_read(address),
            }

            address
        }

        Relative => {
            let rel_address = immediate_fetch(bus, program_counter);

            bus.phantom_read(*program_counter);

            let (address, offset_result) = if rel_address & 0x80 != 0 {
                program_counter.paged_subtract(rel_address)
            } else {
                program_counter.paged_add(rel_address)
            };

            if let OffsetResult::CrossedPage(intermediate) = offset_result {
                bus.phantom_read(intermediate);
            }

            address
        }
    }
}

pub fn address_with_carry<B: Bus>(
    bus: &mut B,
    program_counter: &mut Word,
    address_mode: &AddressMode,
) -> (Word, bool) {
    match address_mode {
        AbsoluteIndexed(index) => {
            let base_address = immediate_fetch_word(bus, program_counter);

            let (address, offset_result) = base_address.paged_add(*index);

            if let OffsetResult::CrossedPage(intermediate) = offset_result {
                bus.phantom_read(intermediate);

                (address, true)
            } else {
                bus.phantom_read(address);

                (address, false)
            }
        }

        _ => panic!(),
    }
}

pub fn data<B: Bus>(bus: &mut B, program_counter: &mut Word, address_mode: &AddressMode) -> u8 {
    match address_mode {
        Immediate => immediate_fetch(bus, program_counter),

        ZeroPage | ZeroPageIndexed(_) | Absolute | IndexedIndirect(_) | Indirect | Relative => {
            let address = address(bus, program_counter, address_mode);

            bus.read(address, CycleOp::CheckInterrupt)
        }

        AbsoluteIndexed(index) => {
            let base_address = immediate_fetch_word(bus, program_counter);

            let (address, offset_result) = base_address.paged_add(*index);

            if let OffsetResult::CrossedPage(intermediate) = offset_result {
                bus.phantom_read(intermediate);
            }

            bus.read(address, CycleOp::CheckInterrupt)
        }

        IndirectIndexed(index) => {
            let zero_page_address = address(bus, program_counter, &ZeroPage);

            let base_address = read_word(bus, zero_page_address, CycleOp::None);

            let (address, offset_result) = base_address.paged_add(*index);

            if let OffsetResult::CrossedPage(intermediate) = offset_result {
                bus.phantom_read(intermediate);
            }

            bus.read(address, CycleOp::CheckInterrupt)
        }
    }
}

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
