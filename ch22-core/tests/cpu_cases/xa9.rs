use rstest_reuse::{self, *};

#[template]
#[rstest]
#[case::case_a9_cc_21(
    Initial{ pc: 45930, s: 172, a: 67, x: 145, y: 150, p: 237, ram: vec![(45930, 169), (45931, 204), (45932, 33)] },
    Expected{ pc: 45932, s: 172, a: 204, x: 145, y: 150, p: 237, cycles: vec![(45930, 169, "read"), (45931, 204, "read")]})
]
pub fn xa9_cases(#[case] initial: Initial, #[case] expected: Expected) {}
