use super::*;

/// The bytes JS reads out of wasm memory (`sound-register-writes.ts`): a
/// little endian u64 base cycle count at 0, a u32 entry count at 8, then 3 byte
/// entries (u16 cycle offset, u8 data) from 12.
#[test]
fn it_lays_out_its_bytes_as_js_reads_them() {
    let mut writes = SoundRegisterWrites::default();
    writes.reset(0x0102_0304_0506_0708);
    writes.push(0x0102_0304_0506_0708 + 0x0a0b, 0xcd);
    writes.push(0x0102_0304_0506_0708 + 0x0c0d, 0xef);

    let bytes = unsafe {
        std::slice::from_raw_parts(
            (&raw const writes).cast::<u8>(),
            size_of::<SoundRegisterWrites>(),
        )
    };

    assert_eq!(
        size_of::<SoundRegisterWrites>(),
        12 + 3 * MAX_SOUND_REG_WRITES
    );
    assert_eq!(
        bytes[..18],
        [
            0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // base cycle count
            0x02, 0x00, 0x00, 0x00, // entry count
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
#[should_panic(expected = "buffer is full")]
fn it_panics_when_the_buffer_is_full() {
    let mut writes = SoundRegisterWrites::default();

    for _ in 0..=MAX_SOUND_REG_WRITES {
        writes.push(0, 0);
    }
}

#[test]
#[should_panic(expected = "offset is too large")]
fn it_panics_when_the_offset_does_not_fit_in_16_bits() {
    let mut writes = SoundRegisterWrites::default();
    writes.reset(1000);

    writes.push(1000 + u64::from(u16::MAX) + 1, 0);
}

#[test]
#[should_panic(expected = "offset is too large")]
fn it_panics_when_a_write_is_before_the_base_cycle_count() {
    let mut writes = SoundRegisterWrites::default();
    writes.reset(1000);

    writes.push(999, 0);
}
