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
    opcode_single_step_tests_from_file("00");
}

#[test]
fn _05_test() {
    opcode_single_step_tests_from_file("05");
}

#[test]
fn _06_test() {
    opcode_single_step_tests_from_file("06");
}

#[test]
fn _08_test() {
    opcode_single_step_tests_from_file("08");
}

#[test]
fn _09_test() {
    opcode_single_step_tests_from_file("09");
}

#[test]
fn _0a_test() {
    opcode_single_step_tests_from_file("0a");
}

#[test]
fn _0d_test() {
    opcode_single_step_tests_from_file("0d");
}

#[test]
fn _0e_test() {
    opcode_single_step_tests_from_file("0e");
}

#[test]
fn _10_test() {
    opcode_single_step_tests_from_file("10");
}

#[test]
fn _18_test() {
    opcode_single_step_tests_from_file("18");
}

#[test]
fn _19_test() {
    opcode_single_step_tests_from_file("19");
}

#[test]
fn _1d_test() {
    opcode_single_step_tests_from_file("1d");
}

#[test]
fn _1e_test() {
    opcode_single_step_tests_from_file("1e");
}

#[test]
fn _20_test() {
    opcode_single_step_tests_from_file("20");
}

#[test]
fn _24_test() {
    opcode_single_step_tests_from_file("24");
}

#[test]
fn _25_test() {
    opcode_single_step_tests_from_file("25");
}

#[test]
fn _26_test() {
    opcode_single_step_tests_from_file("26");
}

#[test]
fn _28_test() {
    opcode_single_step_tests_from_file("28");
}

#[test]
fn _29_test() {
    opcode_single_step_tests_from_file("29");
}

#[test]
fn _2a_test() {
    opcode_single_step_tests_from_file("2a");
}

#[test]
fn _2c_test() {
    opcode_single_step_tests_from_file("2c");
}

#[test]
fn _2d_test() {
    opcode_single_step_tests_from_file("2d");
}

#[test]
fn _2e_test() {
    opcode_single_step_tests_from_file("2e");
}

#[test]
fn _30_test() {
    opcode_single_step_tests_from_file("30");
}

#[test]
fn _31_test() {
    opcode_single_step_tests_from_file("31");
}

#[test]
fn _38_test() {
    opcode_single_step_tests_from_file("38");
}

#[test]
fn _3d_test() {
    opcode_single_step_tests_from_file("3d");
}

#[test]
fn _3e_test() {
    opcode_single_step_tests_from_file("3e");
}

#[test]
fn _40_test() {
    opcode_single_step_tests_from_file("40");
}

#[test]
fn _45_test() {
    opcode_single_step_tests_from_file("45");
}

#[test]
fn _46_test() {
    opcode_single_step_tests_from_file("46");
}

#[test]
fn _48_test() {
    opcode_single_step_tests_from_file("48");
}

#[test]
fn _49_test() {
    opcode_single_step_tests_from_file("49");
}

#[test]
fn _4a_test() {
    opcode_single_step_tests_from_file("4a");
}

#[test]
fn _4c_test() {
    opcode_single_step_tests_from_file("4c");
}

#[test]
fn _4d_test() {
    opcode_single_step_tests_from_file("4d");
}

#[test]
fn _4e_test() {
    opcode_single_step_tests_from_file("4e");
}

#[test]
fn _50_test() {
    opcode_single_step_tests_from_file("50");
}

#[test]
fn _51_test() {
    opcode_single_step_tests_from_file("51");
}

#[test]
fn _58_test() {
    opcode_single_step_tests_from_file("58");
}

#[test]
fn _59_test() {
    opcode_single_step_tests_from_file("59");
}

#[test]
fn _5d_test() {
    opcode_single_step_tests_from_file("5d");
}

#[test]
fn _60_test() {
    opcode_single_step_tests_from_file("60");
}

#[test]
fn _65_test() {
    opcode_single_step_tests_from_file("65");
}

#[test]
fn _66_test() {
    opcode_single_step_tests_from_file("66");
}

#[test]
fn _68_test() {
    opcode_single_step_tests_from_file("68");
}

#[test]
fn _69_test() {
    opcode_single_step_tests_from_file("69");
}

#[test]
fn _6a_test() {
    opcode_single_step_tests_from_file("6a");
}

#[test]
fn _6c_test() {
    opcode_single_step_tests_from_file("6c");
}

#[test]
fn _6d_test() {
    opcode_single_step_tests_from_file("6d");
}

#[test]
fn _6e_test() {
    opcode_single_step_tests_from_file("6e");
}

#[test]
fn _70_test() {
    opcode_single_step_tests_from_file("70");
}

#[test]
fn _71_test() {
    opcode_single_step_tests_from_file("71");
}

#[test]
fn _75_test() {
    opcode_single_step_tests_from_file("75");
}

#[test]
fn _76_test() {
    opcode_single_step_tests_from_file("76");
}

#[test]
fn _78_test() {
    opcode_single_step_tests_from_file("78");
}

#[test]
fn _79_test() {
    opcode_single_step_tests_from_file("79");
}

#[test]
fn _7d_test() {
    opcode_single_step_tests_from_file("7d");
}

#[test]
fn _7e_test() {
    opcode_single_step_tests_from_file("7e");
}

#[test]
fn _81_test() {
    opcode_single_step_tests_from_file("81");
}

#[test]
fn _84_test() {
    opcode_single_step_tests_from_file("84");
}

#[test]
fn _85_test() {
    opcode_single_step_tests_from_file("85");
}

#[test]
fn _86_test() {
    opcode_single_step_tests_from_file("86");
}

#[test]
fn _88_test() {
    opcode_single_step_tests_from_file("88");
}

#[test]
fn _8a_test() {
    opcode_single_step_tests_from_file("8a");
}

#[test]
fn _8c_test() {
    opcode_single_step_tests_from_file("8c");
}

#[test]
fn _8d_test() {
    opcode_single_step_tests_from_file("8d");
}

#[test]
fn _8e_test() {
    opcode_single_step_tests_from_file("8e");
}

#[test]
fn _90_test() {
    opcode_single_step_tests_from_file("90");
}

#[test]
fn _91_test() {
    opcode_single_step_tests_from_file("91");
}

#[test]
fn _94_test() {
    opcode_single_step_tests_from_file("94");
}

#[test]
fn _95_test() {
    opcode_single_step_tests_from_file("95");
}

#[test]
fn _98_test() {
    opcode_single_step_tests_from_file("98");
}

#[test]
fn _99_test() {
    opcode_single_step_tests_from_file("99");
}

#[test]
fn _9a_test() {
    opcode_single_step_tests_from_file("9a");
}

#[test]
fn _9d_test() {
    opcode_single_step_tests_from_file("9d");
}

#[test]
fn _a0_test() {
    opcode_single_step_tests_from_file("a0");
}

#[test]
fn _a1_test() {
    opcode_single_step_tests_from_file("a1");
}

#[test]
fn _a2_test() {
    opcode_single_step_tests_from_file("a2");
}

#[test]
fn _a4_test() {
    opcode_single_step_tests_from_file("a4");
}

#[test]
fn _a5_test() {
    opcode_single_step_tests_from_file("a5");
}

#[test]
fn _a6_test() {
    opcode_single_step_tests_from_file("a6");
}

#[test]
fn _a8_test() {
    opcode_single_step_tests_from_file("a8");
}

#[test]
fn _a9_test() {
    opcode_single_step_tests_from_file("a9");
}

#[test]
fn _aa_test() {
    opcode_single_step_tests_from_file("aa");
}

#[test]
fn _ac_test() {
    opcode_single_step_tests_from_file("ac");
}

#[test]
fn _ad_test() {
    opcode_single_step_tests_from_file("ad");
}

#[test]
fn _ae_test() {
    opcode_single_step_tests_from_file("ae");
}

#[test]
fn _b0_test() {
    opcode_single_step_tests_from_file("b0");
}

#[test]
fn _b1_test() {
    opcode_single_step_tests_from_file("b1");
}

#[test]
fn _b4_test() {
    opcode_single_step_tests_from_file("b4");
}

#[test]
fn _b5_test() {
    opcode_single_step_tests_from_file("b5");
}

#[test]
fn _b8_test() {
    opcode_single_step_tests_from_file("b8");
}

#[test]
fn _b9_test() {
    opcode_single_step_tests_from_file("b9");
}

#[test]
fn _ba_test() {
    opcode_single_step_tests_from_file("ba");
}

#[test]
fn _bc_test() {
    opcode_single_step_tests_from_file("bc");
}

#[test]
fn _bd_test() {
    opcode_single_step_tests_from_file("bd");
}

#[test]
fn _be_test() {
    opcode_single_step_tests_from_file("be");
}

#[test]
fn _c0_test() {
    opcode_single_step_tests_from_file("c0");
}

#[test]
fn _c4_test() {
    opcode_single_step_tests_from_file("c4");
}

#[test]
fn _c5_test() {
    opcode_single_step_tests_from_file("c5");
}

#[test]
fn _c6_test() {
    opcode_single_step_tests_from_file("c6");
}

#[test]
fn _c8_test() {
    opcode_single_step_tests_from_file("c8");
}

#[test]
fn _c9_test() {
    opcode_single_step_tests_from_file("c9");
}

#[test]
fn _ca_test() {
    opcode_single_step_tests_from_file("ca");
}

#[test]
fn _cc_test() {
    opcode_single_step_tests_from_file("cc");
}

#[test]
fn _ce_test() {
    opcode_single_step_tests_from_file("ce");
}

#[test]
fn _cd_test() {
    opcode_single_step_tests_from_file("cd");
}

#[test]
fn _d0_test() {
    opcode_single_step_tests_from_file("d0");
}

#[test]
fn _d1_test() {
    opcode_single_step_tests_from_file("d1");
}

#[test]
fn _d8_test() {
    opcode_single_step_tests_from_file("d8");
}

#[test]
fn _d9_test() {
    opcode_single_step_tests_from_file("d9");
}

#[test]
fn _dd_test() {
    opcode_single_step_tests_from_file("dd");
}

#[test]
fn _de_test() {
    opcode_single_step_tests_from_file("de");
}

#[test]
fn _e0_test() {
    opcode_single_step_tests_from_file("e0");
}

#[test]
fn _e4_test() {
    opcode_single_step_tests_from_file("e4");
}

#[test]
fn _e5_test() {
    opcode_single_step_tests_from_file("e5");
}

#[test]
fn _e6_test() {
    opcode_single_step_tests_from_file("e6");
}

#[test]
fn _e8_test() {
    opcode_single_step_tests_from_file("e8");
}

#[test]
fn _e9_test() {
    opcode_single_step_tests_from_file("e9");
}

#[test]
fn _ec_test() {
    opcode_single_step_tests_from_file("ec");
}

#[test]
fn _ed_test() {
    opcode_single_step_tests_from_file("ed");
}

#[test]
fn _ee_test() {
    opcode_single_step_tests_from_file("ee");
}

#[test]
fn _f0_test() {
    opcode_single_step_tests_from_file("f0");
}

#[test]
fn _f6_test() {
    opcode_single_step_tests_from_file("f6");
}

#[test]
fn _f9_test() {
    opcode_single_step_tests_from_file("f9");
}

#[test]
fn _fd_test() {
    opcode_single_step_tests_from_file("fd");
}

#[test]
fn _fe_test() {
    opcode_single_step_tests_from_file("fe");
}

fn opcode_single_step_tests_from_file(opcode: &str) {
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

    executor.execute();

    assert_eq!(
        &cycle_manager_mock.cycles, expected_cycles,
        "cycles mismatch"
    );

    assert_eq!(registers.pc, final_state.pc, "pc mismatch");
    assert_eq!(registers.s, final_state.s, "s mismatch");
    assert_eq!(registers.a, final_state.a, "a mismatch");
    assert_eq!(registers.x, final_state.x, "x mismatch");
    assert_eq!(registers.y, final_state.y, "y mismatch");
    assert_eq!(registers.get_p(), final_state.p, "p mismatch");
}
