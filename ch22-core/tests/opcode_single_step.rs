mod util;

use ch22_core::cpu::executor::*;
use ch22_core::cpu::registers::*;
use serde::Deserialize;
use std::fs;
use util::{CPUTestState, CycleManagerMock};

type CPUCycles = Vec<(u16, u8, String)>;

#[derive(Deserialize)]
struct SingleStepTestParams {
    name: String,
    initial: CPUTestState,
    r#final: CPUTestState,
    cycles: CPUCycles,
}

#[test]
fn _00_test() {
    opcode_single_step_tests_from_file("00", false);
}

#[test]
fn _01_test() {
    opcode_single_step_tests_from_file("01", false);
}

#[test]
fn _04_test() {
    opcode_single_step_tests_from_file("04", false);
}

#[test]
fn _05_test() {
    opcode_single_step_tests_from_file("05", false);
}

#[test]
fn _06_test() {
    opcode_single_step_tests_from_file("06", false);
}

#[test]
fn _07_test() {
    opcode_single_step_tests_from_file("07", false);
}

#[test]
fn _08_test() {
    opcode_single_step_tests_from_file("08", false);
}

#[test]
fn _09_test() {
    opcode_single_step_tests_from_file("09", false);
}

#[test]
fn _0a_test() {
    opcode_single_step_tests_from_file("0a", false);
}

#[test]
fn _0b_test() {
    opcode_single_step_tests_from_file("0b", false);
}

#[test]
fn _0d_test() {
    opcode_single_step_tests_from_file("0d", false);
}

#[test]
fn _0e_test() {
    opcode_single_step_tests_from_file("0e", false);
}

#[test]
fn _10_test() {
    opcode_single_step_tests_from_file("10", false);
}

#[test]
fn _11_test() {
    opcode_single_step_tests_from_file("11", false);
}

#[test]
fn _15_test() {
    opcode_single_step_tests_from_file("15", false);
}

#[test]
fn _16_test() {
    opcode_single_step_tests_from_file("16", false);
}

#[test]
fn _18_test() {
    opcode_single_step_tests_from_file("18", false);
}

#[test]
fn _19_test() {
    opcode_single_step_tests_from_file("19", false);
}

#[test]
fn _1d_test() {
    opcode_single_step_tests_from_file("1d", false);
}

#[test]
fn _1e_test() {
    opcode_single_step_tests_from_file("1e", false);
}

#[test]
fn _20_test() {
    opcode_single_step_tests_from_file("20", false);
}

#[test]
fn _21_test() {
    opcode_single_step_tests_from_file("21", false);
}

#[test]
fn _24_test() {
    opcode_single_step_tests_from_file("24", false);
}

#[test]
fn _25_test() {
    opcode_single_step_tests_from_file("25", false);
}

#[test]
fn _26_test() {
    opcode_single_step_tests_from_file("26", false);
}

#[test]
fn _28_test() {
    opcode_single_step_tests_from_file("28", false);
}

#[test]
fn _29_test() {
    opcode_single_step_tests_from_file("29", false);
}

#[test]
fn _2a_test() {
    opcode_single_step_tests_from_file("2a", false);
}

#[test]
fn _2c_test() {
    opcode_single_step_tests_from_file("2c", false);
}

#[test]
fn _2d_test() {
    opcode_single_step_tests_from_file("2d", false);
}

#[test]
fn _2e_test() {
    opcode_single_step_tests_from_file("2e", false);
}

#[test]
fn _30_test() {
    opcode_single_step_tests_from_file("30", false);
}

#[test]
fn _31_test() {
    opcode_single_step_tests_from_file("31", false);
}

#[test]
fn _35_test() {
    opcode_single_step_tests_from_file("35", false);
}

#[test]
fn _36_test() {
    opcode_single_step_tests_from_file("36", false);
}

#[test]
fn _38_test() {
    opcode_single_step_tests_from_file("38", false);
}

#[test]
fn _39_test() {
    opcode_single_step_tests_from_file("39", false);
}

#[test]
fn _3d_test() {
    opcode_single_step_tests_from_file("3d", false);
}

#[test]
fn _3e_test() {
    opcode_single_step_tests_from_file("3e", false);
}

#[test]
fn _40_test() {
    opcode_single_step_tests_from_file("40", false);
}

#[test]
fn _41_test() {
    opcode_single_step_tests_from_file("41", false);
}

#[test]
fn _45_test() {
    opcode_single_step_tests_from_file("45", false);
}

#[test]
fn _46_test() {
    opcode_single_step_tests_from_file("46", false);
}

#[test]
fn _48_test() {
    opcode_single_step_tests_from_file("48", false);
}

#[test]
fn _49_test() {
    opcode_single_step_tests_from_file("49", false);
}

#[test]
fn _4a_test() {
    opcode_single_step_tests_from_file("4a", false);
}

#[test]
fn _4b_test() {
    opcode_single_step_tests_from_file("4b", false);
}

#[test]
fn _4c_test() {
    opcode_single_step_tests_from_file("4c", false);
}

#[test]
fn _4d_test() {
    opcode_single_step_tests_from_file("4d", false);
}

#[test]
fn _4e_test() {
    opcode_single_step_tests_from_file("4e", false);
}

#[test]
fn _50_test() {
    opcode_single_step_tests_from_file("50", false);
}

#[test]
fn _51_test() {
    opcode_single_step_tests_from_file("51", false);
}

#[test]
fn _55_test() {
    opcode_single_step_tests_from_file("55", false);
}

#[test]
fn _56_test() {
    opcode_single_step_tests_from_file("56", false);
}

#[test]
fn _58_test() {
    opcode_single_step_tests_from_file("58", false);
}

#[test]
fn _59_test() {
    opcode_single_step_tests_from_file("59", false);
}

#[test]
fn _5d_test() {
    opcode_single_step_tests_from_file("5d", false);
}

#[test]
fn _60_test() {
    opcode_single_step_tests_from_file("60", false);
}

#[test]
fn _61_test() {
    opcode_single_step_tests_from_file("61", false);
}

#[test]
fn _65_test() {
    opcode_single_step_tests_from_file("65", false);
}

#[test]
fn _66_test() {
    opcode_single_step_tests_from_file("66", false);
}

#[test]
fn _68_test() {
    opcode_single_step_tests_from_file("68", false);
}

#[test]
fn _69_test() {
    opcode_single_step_tests_from_file("69", false);
}

#[test]
fn _6a_test() {
    opcode_single_step_tests_from_file("6a", false);
}

#[test]
fn _6c_test() {
    opcode_single_step_tests_from_file("6c", false);
}

#[test]
fn _6d_test() {
    opcode_single_step_tests_from_file("6d", false);
}

#[test]
fn _6e_test() {
    opcode_single_step_tests_from_file("6e", false);
}

#[test]
fn _70_test() {
    opcode_single_step_tests_from_file("70", false);
}

#[test]
fn _71_test() {
    opcode_single_step_tests_from_file("71", false);
}

#[test]
fn _75_test() {
    opcode_single_step_tests_from_file("75", false);
}

#[test]
fn _76_test() {
    opcode_single_step_tests_from_file("76", false);
}

#[test]
fn _78_test() {
    opcode_single_step_tests_from_file("78", false);
}

#[test]
fn _79_test() {
    opcode_single_step_tests_from_file("79", false);
}

#[test]
fn _7d_test() {
    opcode_single_step_tests_from_file("7d", false);
}

#[test]
fn _7e_test() {
    opcode_single_step_tests_from_file("7e", false);
}

#[test]
fn _81_test() {
    opcode_single_step_tests_from_file("81", false);
}

#[test]
fn _84_test() {
    opcode_single_step_tests_from_file("84", false);
}

#[test]
fn _85_test() {
    opcode_single_step_tests_from_file("85", false);
}

#[test]
fn _86_test() {
    opcode_single_step_tests_from_file("86", false);
}

#[test]
fn _87_test() {
    opcode_single_step_tests_from_file("87", false);
}

#[test]
fn _88_test() {
    opcode_single_step_tests_from_file("88", false);
}

#[test]
fn _8a_test() {
    opcode_single_step_tests_from_file("8a", false);
}

#[test]
fn _8c_test() {
    opcode_single_step_tests_from_file("8c", false);
}

#[test]
fn _8d_test() {
    opcode_single_step_tests_from_file("8d", false);
}

#[test]
fn _8e_test() {
    opcode_single_step_tests_from_file("8e", false);
}

#[test]
fn _90_test() {
    opcode_single_step_tests_from_file("90", false);
}

#[test]
fn _91_test() {
    opcode_single_step_tests_from_file("91", false);
}

#[test]
fn _94_test() {
    opcode_single_step_tests_from_file("94", false);
}

#[test]
fn _95_test() {
    opcode_single_step_tests_from_file("95", false);
}

#[test]
fn _96_test() {
    opcode_single_step_tests_from_file("96", false);
}

#[test]
fn _98_test() {
    opcode_single_step_tests_from_file("98", false);
}

#[test]
fn _99_test() {
    opcode_single_step_tests_from_file("99", false);
}

#[test]
fn _9a_test() {
    opcode_single_step_tests_from_file("9a", false);
}

#[test]
fn _9c_test() {
    opcode_single_step_tests_from_file("9c", true);
}

#[test]
fn _9d_test() {
    opcode_single_step_tests_from_file("9d", false);
}

#[test]
fn _a0_test() {
    opcode_single_step_tests_from_file("a0", false);
}

#[test]
fn _a1_test() {
    opcode_single_step_tests_from_file("a1", false);
}

#[test]
fn _a2_test() {
    opcode_single_step_tests_from_file("a2", false);
}

#[test]
fn _a4_test() {
    opcode_single_step_tests_from_file("a4", false);
}

#[test]
fn _a5_test() {
    opcode_single_step_tests_from_file("a5", false);
}

#[test]
fn _a6_test() {
    opcode_single_step_tests_from_file("a6", false);
}

#[test]
fn _a8_test() {
    opcode_single_step_tests_from_file("a8", false);
}

#[test]
fn _a9_test() {
    opcode_single_step_tests_from_file("a9", false);
}

#[test]
fn _aa_test() {
    opcode_single_step_tests_from_file("aa", false);
}

#[test]
fn _ac_test() {
    opcode_single_step_tests_from_file("ac", false);
}

#[test]
fn _ad_test() {
    opcode_single_step_tests_from_file("ad", false);
}

#[test]
fn _ae_test() {
    opcode_single_step_tests_from_file("ae", false);
}

#[test]
fn _b0_test() {
    opcode_single_step_tests_from_file("b0", false);
}

#[test]
fn _b1_test() {
    opcode_single_step_tests_from_file("b1", false);
}

#[test]
fn _b4_test() {
    opcode_single_step_tests_from_file("b4", false);
}

#[test]
fn _b5_test() {
    opcode_single_step_tests_from_file("b5", false);
}

#[test]
fn _b6_test() {
    opcode_single_step_tests_from_file("b6", false);
}

#[test]
fn _b8_test() {
    opcode_single_step_tests_from_file("b8", false);
}

#[test]
fn _b9_test() {
    opcode_single_step_tests_from_file("b9", false);
}

#[test]
fn _ba_test() {
    opcode_single_step_tests_from_file("ba", false);
}

#[test]
fn _bc_test() {
    opcode_single_step_tests_from_file("bc", false);
}

#[test]
fn _bd_test() {
    opcode_single_step_tests_from_file("bd", false);
}

#[test]
fn _be_test() {
    opcode_single_step_tests_from_file("be", false);
}

#[test]
fn _c0_test() {
    opcode_single_step_tests_from_file("c0", false);
}

#[test]
fn _c1_test() {
    opcode_single_step_tests_from_file("c1", false);
}

#[test]
fn _c4_test() {
    opcode_single_step_tests_from_file("c4", false);
}

#[test]
fn _c5_test() {
    opcode_single_step_tests_from_file("c5", false);
}

#[test]
fn _c6_test() {
    opcode_single_step_tests_from_file("c6", false);
}

#[test]
fn _c8_test() {
    opcode_single_step_tests_from_file("c8", false);
}

#[test]
fn _c9_test() {
    opcode_single_step_tests_from_file("c9", false);
}

#[test]
fn _ca_test() {
    opcode_single_step_tests_from_file("ca", false);
}

#[test]
fn _cc_test() {
    opcode_single_step_tests_from_file("cc", false);
}

#[test]
fn _cd_test() {
    opcode_single_step_tests_from_file("cd", false);
}

#[test]
fn _ce_test() {
    opcode_single_step_tests_from_file("ce", false);
}

#[test]
fn _d0_test() {
    opcode_single_step_tests_from_file("d0", false);
}

#[test]
fn _d1_test() {
    opcode_single_step_tests_from_file("d1", false);
}

#[test]
fn _d5_test() {
    opcode_single_step_tests_from_file("d5", false);
}

#[test]
fn _d6_test() {
    opcode_single_step_tests_from_file("d6", false);
}

#[test]
fn _d8_test() {
    opcode_single_step_tests_from_file("d8", false);
}

#[test]
fn _d9_test() {
    opcode_single_step_tests_from_file("d9", false);
}

#[test]
fn _dc_test() {
    opcode_single_step_tests_from_file("dc", true);
}

#[test]
fn _dd_test() {
    opcode_single_step_tests_from_file("dd", false);
}

#[test]
fn _de_test() {
    opcode_single_step_tests_from_file("de", false);
}

#[test]
fn _e0_test() {
    opcode_single_step_tests_from_file("e0", false);
}

#[test]
fn _e4_test() {
    opcode_single_step_tests_from_file("e4", false);
}

#[test]
fn _e5_test() {
    opcode_single_step_tests_from_file("e5", false);
}

#[test]
fn _e6_test() {
    opcode_single_step_tests_from_file("e6", false);
}

#[test]
fn _e8_test() {
    opcode_single_step_tests_from_file("e8", false);
}

#[test]
fn _e9_test() {
    opcode_single_step_tests_from_file("e9", false);
}

#[test]
fn _ea_test() {
    opcode_single_step_tests_from_file("ea", false);
}

#[test]
fn _ec_test() {
    opcode_single_step_tests_from_file("ec", false);
}

#[test]
fn _ed_test() {
    opcode_single_step_tests_from_file("ed", false);
}

#[test]
fn _ee_test() {
    opcode_single_step_tests_from_file("ee", false);
}

#[test]
fn _f0_test() {
    opcode_single_step_tests_from_file("f0", false);
}

#[test]
fn _f1_test() {
    opcode_single_step_tests_from_file("f1", false);
}

#[test]
fn _f5_test() {
    opcode_single_step_tests_from_file("f5", false);
}

#[test]
fn _f6_test() {
    opcode_single_step_tests_from_file("f6", false);
}

#[test]
fn _f8_test() {
    opcode_single_step_tests_from_file("f8", false);
}

#[test]
fn _f9_test() {
    opcode_single_step_tests_from_file("f9", false);
}

#[test]
fn _fd_test() {
    opcode_single_step_tests_from_file("fd", false);
}

#[test]
fn _fe_test() {
    opcode_single_step_tests_from_file("fe", false);
}

fn opcode_single_step_tests_from_file(opcode: &str, ignore_break: bool) {
    let data = fs::read_to_string(format!(
        "./tests/single_step_tests_65x02/6502/v1/{opcode}.json"
    ))
    .expect("Unable to read file");

    let test_params: Vec<SingleStepTestParams> =
        serde_json::from_str(&data).expect("JSON was not well-formatted");

    for test_param in &test_params {
        let panics = std::panic::catch_unwind(|| {
            opcode_single_step_test(
                &test_param.name,
                &test_param.initial,
                &test_param.r#final,
                &test_param.cycles,
                ignore_break,
            )
        });

        if panics.is_err() {
            panic!("error was in {:?}", test_param.name)
        }
    }
}

fn opcode_single_step_test(
    _name: &str,
    initial_state: &CPUTestState,
    final_state: &CPUTestState,
    expected_cycles: &CPUCycles,
    ignore_break: bool,
) {
    let mut registers = Registers::new();
    registers.pc = initial_state.pc;
    registers.s = initial_state.s;
    registers.a = initial_state.a;
    registers.x = initial_state.x;
    registers.y = initial_state.y;
    registers.set_p(initial_state.p);

    let mut cycle_manager_mock = CycleManagerMock::new(&initial_state.ram);

    let mut executor = Executor::new(&mut cycle_manager_mock, &mut registers);

    executor.execute(true);

    assert_eq!(
        &cycle_manager_mock.cycles, expected_cycles,
        "cycles mismatch"
    );

    assert_eq!(registers.pc, final_state.pc, "pc mismatch");
    assert_eq!(registers.s, final_state.s, "s mismatch");
    assert_eq!(registers.a, final_state.a, "a mismatch");
    assert_eq!(registers.x, final_state.x, "x mismatch");
    assert_eq!(registers.y, final_state.y, "y mismatch");

    let expected_p = if ignore_break {
        final_state.p & !P_BREAK_FLAG
    } else {
        final_state.p
    };
    assert_eq!(registers.get_p(), expected_p, "p mismatch");
}
