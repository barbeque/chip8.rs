pub fn get_hex_font() -> Vec<u8> {
    let mut result = Vec::<u8>::with_capacity(16);

    // 0
    result.push(0xf0);
    result.push(0x90);
    result.push(0x90);
    result.push(0x90);
    result.push(0xf0);

    // 1
    result.push(0x20);
    result.push(0x60);
    result.push(0x20);
    result.push(0x20);
    result.push(0x70);

    // 2
    result.push(0xf0);
    result.push(0x10);
    result.push(0xf0);
    result.push(0x80);
    result.push(0xf0);

    // 3
    result.push(0xf0);
    result.push(0x10);
    result.push(0xf0);
    result.push(0x10);
    result.push(0xf0);

    // 4
    result.push(0x90);
    result.push(0x90);
    result.push(0xf0);
    result.push(0x10);
    result.push(0x10);

    // 5
    result.push(0xf0);
    result.push(0x80);
    result.push(0xf0);
    result.push(0x10);
    result.push(0xf0);

    // 6
    result.push(0xf0);
    result.push(0x80);
    result.push(0xf0);
    result.push(0x90);
    result.push(0xf0);

    // 7
    result.push(0xf0);
    result.push(0x10);
    result.push(0x20);
    result.push(0x40);
    result.push(0x40);

    // 8
    result.push(0xf0);
    result.push(0x90);
    result.push(0xf0);
    result.push(0x90);
    result.push(0xf0);

    // 9
    result.push(0xf0);
    result.push(0x90);
    result.push(0xf0);
    result.push(0x10);
    result.push(0xf0);

    // A
    result.push(0xf0);
    result.push(0x90);
    result.push(0xf0);
    result.push(0x90);
    result.push(0x90);

    // B
    result.push(0xe0);
    result.push(0x90);
    result.push(0xe0);
    result.push(0x90);
    result.push(0xe0);

    // C
    result.push(0xf0);
    result.push(0x80);
    result.push(0x80);
    result.push(0x80);
    result.push(0xf0);

    // D
    result.push(0xe0);
    result.push(0x90);
    result.push(0x90);
    result.push(0x90);
    result.push(0xe0);

    // E
    result.push(0xf0);
    result.push(0x80);
    result.push(0xf0);
    result.push(0x80);
    result.push(0xf0);

    // F
    result.push(0xf0);
    result.push(0x80);
    result.push(0xf0);
    result.push(0x80);
    result.push(0x80);

    result
}

#[cfg(test)]
mod font_tests {
    use super::*;

    #[test]
    fn length_makes_sense() {
        let font = get_hex_font();
        assert_eq!(font.len(), 16 * 5);
    }

    #[test]
    fn characters_right_width() {
        // Make sure none of them are more than 4 'pixels' wide
        let font = get_hex_font();
        for i in 0..font.len() {
            assert!(font[i] <= 0xf0);
        }
    }
}
