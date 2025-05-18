use super::memory_util::*;
use crate::cpu::interrupt_state::*;
use crate::cpu_io::*;
use crate::word::*;

use AddressMode::*;

pub fn get_address<IO: CpuIO>(
    io: &mut IO,
    program_counter: &mut Word,
    address_mode: &AddressMode,
) -> Word {
    match address_mode {
        Immediate => panic!(),

        ZeroPage => Word::zero_page(immediate_fetch(io, program_counter)),

        ZeroPageIndexed(index) => {
            let base_address = Word::zero_page(immediate_fetch(io, program_counter));

            io.phantom_read(base_address);

            base_address.same_page_add(*index)
        }

        Absolute => immediate_fetch_word(io, program_counter),

        AbsoluteIndexed(index) => {
            let base_address = immediate_fetch_word(io, program_counter);

            let (address, offset_result) = base_address.paged_add(*index);

            match offset_result {
                OffsetResult::CrossedPage(intermediate) => io.phantom_read(intermediate),

                OffsetResult::SamePage => io.phantom_read(address),
            }

            address
        }

        Indirect => {
            let base_address = immediate_fetch_word(io, program_counter);

            read_word(io, base_address)
        }

        IndexedIndirect(index) => {
            let address = get_address(io, program_counter, &ZeroPageIndexed(*index));

            read_word(io, address)
        }

        IndirectIndexed(index) => {
            let zero_page_address = get_address(io, program_counter, &ZeroPage);

            let base_address = read_word(io, zero_page_address);

            let (address, offset_result) = base_address.paged_add(*index);

            match offset_result {
                OffsetResult::CrossedPage(intermediate) => io.phantom_read(intermediate),

                OffsetResult::SamePage => io.phantom_read(address),
            }

            address
        }

        _ => panic!(),
    }
}

pub fn get_address_with_interrupt_check<IO: CpuIO>(
    io: &mut IO,
    program_counter: &mut Word,
    address_mode: &AddressMode,
    interrupt_disable: bool,
    interrupt_state: &mut InterruptState,
) -> Word {
    match address_mode {
        Absolute => immediate_fetch_word_with_interrupt_check(
            io,
            program_counter,
            interrupt_disable,
            interrupt_state,
        ),

        Indirect => {
            let base_address = immediate_fetch_word(io, program_counter);

            read_word_with_interrupt_check(io, base_address, interrupt_disable, interrupt_state)
        }

        Relative => {
            let rel_address = immediate_fetch(io, program_counter);

            io.phantom_read(*program_counter);

            let (address, offset_result) = if rel_address & 0x80 != 0 {
                program_counter.paged_subtract(rel_address)
            } else {
                program_counter.paged_add(rel_address)
            };

            if let OffsetResult::CrossedPage(intermediate) = offset_result {
                update_interrupt_state(interrupt_state, io, interrupt_disable);

                io.phantom_read(intermediate);
            }

            address
        }

        _ => panic!(),
    }
}

pub fn address_with_carry<IO: CpuIO>(
    io: &mut IO,
    program_counter: &mut Word,
    address_mode: &AddressMode,
) -> (Word, bool) {
    match address_mode {
        AbsoluteIndexed(index) => {
            let base_address = immediate_fetch_word(io, program_counter);

            let (address, offset_result) = base_address.paged_add(*index);

            if let OffsetResult::CrossedPage(intermediate) = offset_result {
                io.phantom_read(intermediate);

                (address, true)
            } else {
                io.phantom_read(address);

                (address, false)
            }
        }

        _ => panic!(),
    }
}

pub fn get_data_with_interrupt_check<IO: CpuIO>(
    io: &mut IO,
    program_counter: &mut Word,
    address_mode: &AddressMode,
    interrupt_disable: bool,
    interrupt_state: &mut InterruptState,
) -> u8 {
    match address_mode {
        Immediate => {
            update_interrupt_state(interrupt_state, io, interrupt_disable);

            immediate_fetch(io, program_counter)
        }

        ZeroPage | ZeroPageIndexed(_) | Absolute | IndexedIndirect(_) | Indirect => {
            let address = get_address(io, program_counter, address_mode);

            update_interrupt_state(interrupt_state, io, interrupt_disable);

            io.read(address)
        }

        AbsoluteIndexed(index) => {
            let base_address = immediate_fetch_word(io, program_counter);

            let (address, offset_result) = base_address.paged_add(*index);

            if let OffsetResult::CrossedPage(intermediate) = offset_result {
                io.phantom_read(intermediate);
            }

            update_interrupt_state(interrupt_state, io, interrupt_disable);

            io.read(address)
        }

        IndirectIndexed(index) => {
            let zero_page_address = get_address(io, program_counter, &ZeroPage);

            let base_address = read_word(io, zero_page_address);

            let (address, offset_result) = base_address.paged_add(*index);

            if let OffsetResult::CrossedPage(intermediate) = offset_result {
                io.phantom_read(intermediate);
            }

            update_interrupt_state(interrupt_state, io, interrupt_disable);

            io.read(address)
        }

        _ => panic!(),
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
