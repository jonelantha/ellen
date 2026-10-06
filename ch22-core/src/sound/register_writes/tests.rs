use super::*;

/// The bytes the host reads out of wasm memory: a little endian u64 base cycle
/// count at 0, a u32 entry count at 8, a u8 of dropped flags at 12, then 3 byte
/// entries (u16 cycle offset, u8 data) from 13.
#[test]
fn it_lays_out_its_bytes_as_the_host_reads_them() {
    let mut writes = SoundRegisterWrites::default();
    writes.reset(0x0102_0304_0506_0708);
    writes.push(0x0102_0304_0506_0708 + 0x0a0b, 0xcd);
    writes.push(0x0102_0304_0506_0708 + 0x0c0d, 0xef);
    writes.push(0x0102_0304_0506_0708 + 0x1_0000, 0x99); // offset too large

    let bytes = unsafe {
        std::slice::from_raw_parts(
            (&raw const writes).cast::<u8>(),
            size_of::<SoundRegisterWrites>(),
        )
    };

    assert_eq!(
        size_of::<SoundRegisterWrites>(),
        13 + 3 * MAX_SOUND_REG_WRITES
    );
    assert_eq!(
        bytes[..19],
        [
            0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // base cycle count
            0x02, 0x00, 0x00, 0x00, // entry count
            0x02, // dropped flags
            0x0b, 0x0a, 0xcd, // first entry
            0x0d, 0x0c, 0xef, // second entry
        ]
    );
}

#[test]
fn it_records_offsets_relative_to_the_base_cycle_count() {
    let mut writes = SoundRegisterWrites::default();
    writes.reset(1000);

    writes.push(1000, 1);
    writes.push(1005, 2);
    writes.push(1000 + u64::from(u16::MAX), 3);

    let entries: Vec<(u16, u8)> = (0..3)
        .map(|index| {
            let entry = &writes.entries[index];
            (entry.cycle_offset, entry.data)
        })
        .collect();

    assert_eq!(entries, [(0, 1), (5, 2), (u16::MAX, 3)]);
    assert_eq!({ writes.num_entries }, 3);
}

#[test]
fn it_empties_and_rebases_on_reset() {
    let mut writes = SoundRegisterWrites::default();
    writes.reset(1000);
    writes.push(1010, 1);

    writes.reset(2000);

    assert_eq!({ writes.num_entries }, 0);
    assert_eq!({ writes.base_cycle_count }, 2000);
}

#[test]
fn it_holds_exactly_the_maximum_number_of_writes() {
    let mut writes = SoundRegisterWrites::default();

    for _ in 0..MAX_SOUND_REG_WRITES {
        writes.push(0, 0);
    }

    assert_eq!({ writes.num_entries } as usize, MAX_SOUND_REG_WRITES);
}

#[test]
fn it_ignores_and_flags_a_write_when_the_buffer_is_full() {
    let mut writes = SoundRegisterWrites::default();

    for index in 0..MAX_SOUND_REG_WRITES {
        writes.push(0, index as u8);
    }

    writes.push(0, 0xee);

    assert_eq!({ writes.num_entries } as usize, MAX_SOUND_REG_WRITES);
    assert_eq!(writes.cycles_and_data().last(), Some(&(0, 0xf3)));
    assert_eq!({ writes.dropped }, DROPPED_BUFFER_FULL);
}

#[test]
fn it_ignores_and_flags_a_write_whose_offset_does_not_fit_in_16_bits() {
    let mut writes = SoundRegisterWrites::default();
    writes.reset(1000);

    writes.push(1000 + u64::from(u16::MAX) + 1, 0);

    assert_eq!({ writes.num_entries }, 0);
    assert_eq!({ writes.dropped }, DROPPED_OFFSET_TOO_LARGE);
}

#[test]
fn it_keeps_recording_after_an_ignored_write() {
    let mut writes = SoundRegisterWrites::default();
    writes.reset(1000);

    writes.push(1000 + u64::from(u16::MAX) + 1, 1);
    writes.push(1010, 2);

    assert_eq!(writes.cycles_and_data(), [(1010, 2)]);
}

#[test]
fn it_flags_both_reasons_and_clears_the_flags_on_reset() {
    let mut writes = SoundRegisterWrites::default();
    writes.reset(1000);

    for _ in 0..MAX_SOUND_REG_WRITES {
        writes.push(1000, 0);
    }
    writes.push(1000, 0);
    writes.push(1000 + u64::from(u16::MAX) + 1, 0);

    assert_eq!(
        { writes.dropped },
        DROPPED_BUFFER_FULL | DROPPED_OFFSET_TOO_LARGE
    );

    writes.reset(2000);

    assert_eq!({ writes.dropped }, 0);
}
