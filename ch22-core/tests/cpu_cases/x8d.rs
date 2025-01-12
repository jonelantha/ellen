use rstest_reuse::{self, *};

#[template]
#[rstest]
#[case::case_8d_0d_22(
    Initial{ pc: 14188, s: 115, a: 253, x: 229, y: 196, p: 101, ram: vec![ (14188, 141), (14189, 13), (14190, 34), (14191, 230)] },
    Expected{ pc: 14191, s: 115, a: 253, x: 229, y: 196, p: 101, cycles: vec![ (14188, 141, "read"), (14189, 13, "read"), (14190, 34, "read"), (8717, 253, "write")]}
)]
pub fn x8d_cases(#[case] initial: Initial, #[case] expected: Expected) {}
