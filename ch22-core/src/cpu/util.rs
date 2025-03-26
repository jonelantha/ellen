// helpers

pub fn is_negative(value: u8) -> bool {
    value & 0x80 != 0
}

// nibble helpers

pub fn wrap_nibble_up(nibble: u8) -> (u8, u8) {
    if nibble > 0x09 {
        (nibble + 0x06, 1)
    } else {
        (nibble, 0)
    }
}

pub fn wrap_nibble_down(nibble: u8) -> (u8, u8) {
    if nibble & 0x10 != 0 {
        (nibble - 0x06, 1)
    } else {
        (nibble, 0)
    }
}

pub fn from_nibbles(high: u8, low: u8) -> u8 {
    (high << 4) | (low & 0x0f)
}

pub fn to_high_nibble(value: u8) -> u8 {
    value >> 4
}

pub fn to_low_nibble(value: u8) -> u8 {
    value & 0x0f
}
