use super::*;

// The level of the /WE line, which is active low.
const WE_HIGH: bool = true;
const WE_LOW: bool = false;

#[test]
fn it_records_one_write_per_we_fall_plus_one_per_data_change_while_we_is_low() {
    // (description, [(we level, data, cycles)], expected [(cycles, data)])
    let test_cases = [
        (
            "a fall",
            vec![(WE_HIGH, 0x12, 10), (WE_LOW, 0xab, 20)],
            vec![(20, 0xab)],
        ),
        (
            "low from the start",
            vec![(WE_LOW, 0xff, 0)],
            vec![(0, 0xff)],
        ),
        (
            "held low with the same data",
            vec![(WE_LOW, 0xab, 10), (WE_LOW, 0xab, 20), (WE_LOW, 0xab, 30)],
            vec![(10, 0xab)],
        ),
        (
            "data changing while low",
            vec![(WE_LOW, 0xab, 10), (WE_LOW, 0xcd, 20)],
            vec![(10, 0xab), (20, 0xcd)],
        ),
        (
            "a rise",
            vec![(WE_LOW, 0xab, 10), (WE_HIGH, 0xab, 20)],
            vec![(10, 0xab)],
        ),
        (
            "data changing while high",
            vec![(WE_HIGH, 0x01, 10), (WE_HIGH, 0x02, 20)],
            vec![],
        ),
        (
            "the same data again after a rise",
            vec![(WE_LOW, 0xab, 10), (WE_HIGH, 0xab, 20), (WE_LOW, 0xab, 30)],
            vec![(10, 0xab), (30, 0xab)],
        ),
    ];

    for (description, updates, expected_writes) in test_cases {
        let mut recorder = SoundRegisterWriteRecorder::default();
        recorder.start_field(0);

        let latched_count = updates
            .into_iter()
            .filter(|&(we, data, cycles)| recorder.update(we, data, cycles))
            .count();

        assert_eq!(recorded_writes(&recorder), expected_writes, "{description}");
        assert_eq!(latched_count, expected_writes.len(), "{description}");
    }
}

#[test]
fn it_carries_the_input_state_across_fields() {
    let mut recorder = SoundRegisterWriteRecorder::default();

    recorder.start_field(0);
    recorder.update(WE_LOW, 0xab, 10);

    recorder.start_field(1000);

    // /WE has been low the whole time, so nothing is written again
    assert!(!recorder.update(WE_LOW, 0xab, 1010));
    assert!(recorder.update(WE_LOW, 0xcd, 1020));
    assert_eq!(recorded_writes(&recorder), [(1020, 0xcd)]);
}

#[test]
fn it_empties_the_buffer_and_rebases_offsets_when_a_field_starts() {
    let mut recorder = SoundRegisterWriteRecorder::default();

    recorder.start_field(0);
    recorder.update(WE_LOW, 0xab, 10);

    recorder.start_field(1000);

    assert_eq!(recorded_writes(&recorder), []);

    recorder.update(WE_HIGH, 0xab, 1002);
    recorder.update(WE_LOW, 0x01, 1005);

    let writes = recorder.register_writes();
    let base_cycle_count = writes.base_cycle_count;
    let cycle_offset = writes.entries[0].cycle_offset;

    assert_eq!(base_cycle_count, 1000);
    assert_eq!(cycle_offset, 5);
}

fn recorded_writes(recorder: &SoundRegisterWriteRecorder) -> Vec<(u64, u8)> {
    let writes = recorder.register_writes();
    let base_cycle_count = writes.base_cycle_count;

    writes.entries[..writes.num_entries]
        .iter()
        .map(|entry| (base_cycle_count + u64::from(entry.cycle_offset), entry.data))
        .collect()
}
