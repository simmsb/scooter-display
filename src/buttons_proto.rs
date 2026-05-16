#[derive(deku::DekuRead, deku::DekuWrite, Copy, Clone, Debug, PartialEq)]
#[deku(bit_order = "lsb")]
pub struct ButtonParser {
    #[deku(bits = 1)]
    pub down_pressed: bool,

    #[deku(bits = 1)]
    pub up_pressed: bool,

    #[deku(bits = 1)]
    pub r_pressed: bool,

    #[deku(bits = 1)]
    pub l_pressed: bool,

    #[deku(bits = 1)]
    pub r_blink: bool,

    #[deku(pad_bits_after = "2", bits = 1)]
    pub l_blink: bool,

    #[deku(pad_bits_after = "7", bits = 1)]
    pub confirm_pressed: bool,
}

defmt::bitflags! {
    pub struct Buttons: u8 {
        const CONFIRM = 1 << 0;
        const UP      = 1 << 1;
        const DOWN    = 1 << 2;
        const L       = 1 << 3;
        const R       = 1 << 4;
        const L_BLINK = 1 << 5;
        const R_BLINK = 1 << 6;
        const POWER   = 1 << 7;
    }
}

impl Buttons {
    pub fn update_from_uart(&mut self, parsed: ButtonParser) {
        self.set(Buttons::CONFIRM, parsed.confirm_pressed);
        self.set(Buttons::UP, parsed.up_pressed);
        self.set(Buttons::DOWN, parsed.down_pressed);
        self.set(Buttons::L, parsed.l_pressed);
        self.set(Buttons::R, parsed.r_pressed);
        self.set(Buttons::L_BLINK, parsed.l_blink);
        self.set(Buttons::R_BLINK, parsed.r_blink);
    }
}

#[cfg(test)]
mod test {
    use deku::DekuContainerWrite as _;

    use super::*;

    #[test]
    fn button_parser() {
        let mut tmp = [0u8; 2];
        ButtonParser {
            confirm_pressed: true,
            up_pressed: true,
            down_pressed: false,
            l_pressed: false,
            r_pressed: true,
            r_blink: false,
            l_blink: false,
        }
        .to_slice(&mut tmp)
        .unwrap();

        assert_eq!(tmp, [6, 1]);

        let serialized = &[0b0000_0011, 0b000000001];

        let parsed = ButtonParser::try_from(serialized.as_slice()).unwrap();

        assert_eq!(
            parsed,
            ButtonParser {
                confirm_pressed: true,
                up_pressed: true,
                down_pressed: true,
                l_pressed: false,
                r_pressed: false,
                r_blink: false,
                l_blink: false,
            }
        )
    }
}
