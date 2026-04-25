#[derive(deku::DekuRead, deku::DekuWrite, Copy, Clone, Debug, PartialEq)]
pub struct ButtonParser {
    #[deku(pad_bits_before = "4", bits = 1)]
    pub l_pressed: bool,

    #[deku(bits = 1)]
    pub down_pressed: bool,

    #[deku(bits = 1)]
    pub up_pressed: bool,

    #[deku(bits = 1)]
    pub confirm_pressed: bool,

    #[deku(pad_bits_before = "7", bits = 1)]
    pub r_pressed: bool,
}

impl ButtonParser {
    pub fn as_buttons(self) -> Buttons {
        let mut b = Buttons::empty();
        b.set(Buttons::CONFIRM, self.confirm_pressed);
        b.set(Buttons::UP, self.up_pressed);
        b.set(Buttons::DOWN, self.down_pressed);
        b.set(Buttons::L, self.l_pressed);
        b.set(Buttons::R, self.r_pressed);
        b
    }
}

defmt::bitflags! {
    pub struct Buttons: u8 {
        const CONFIRM = 0b00000001;
        const UP      = 0b00000010;
        const DOWN    = 0b00000100;
        const L       = 0b00001000;
        const R       = 0b00010000;
        const MAIN    = 0b00100000;
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
        }
        .to_slice(&mut tmp)
        .unwrap();

        assert_eq!(tmp, [3, 1]);

        let serialized = &[0b0000_0011, 0b000000001];

        let parsed = ButtonParser::try_from(serialized.as_slice()).unwrap();

        assert_eq!(
            parsed,
            ButtonParser {
                confirm_pressed: true,
                up_pressed: true,
                down_pressed: false,
                l_pressed: false,
                r_pressed: true
            }
        )
    }
}
