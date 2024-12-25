const NUM_KEYS: usize = 128;

pub struct InputMonitoring {
    pub keyboard: Keyboard,
    pub mouse: Mouse,
}

impl InputMonitoring {
    pub fn clear_keyboard(&mut self) {
        for i in 0..NUM_KEYS {
            self.keyboard.buttons[i] = false;
        }
    }

    // convenience method, returns true if any keyboard key was pressed
    pub fn any_key_pressed(&self) -> bool {
        for i in 0..NUM_KEYS {
            if self.keyboard.buttons[i] == true {
                return true;
            }
        }
        false
    }

    pub fn key_pressed(&self, key: NumCode) -> bool {
        return self.keyboard.buttons[key as usize];
    }
}

// TODO convert to InputMonitoring::new()
pub fn new_input_monitoring() -> InputMonitoring {
    let keyboard = Keyboard {
        last_scan: NumCode::None,
        last_ascii: '\0',
        buttons: [false; NUM_KEYS],
    };
    let mouse = Mouse {};

    InputMonitoring { keyboard, mouse }
}

pub struct Keyboard {
    pub last_scan: NumCode,
    pub last_ascii: char,
    pub buttons: [bool; NUM_KEYS],
}

impl Keyboard {
    pub fn update_last_value(&mut self, last_scan: NumCode) {
        self.last_scan = last_scan;
        self.update_ascii(last_scan);
    }

    fn update_ascii(&mut self, last_scan: NumCode) {
        // TODO handle CapsLock (requires state in the Keyboard mgr)
        let shifted =
            self.buttons[NumCode::LShift as usize] || self.buttons[NumCode::RShift as usize];
        let c = match (last_scan, shifted) {
            (NumCode::Num0, false) | (NumCode::Num0, true) => '0',
            (NumCode::Num1, false) | (NumCode::Num1, true) => '1',
            (NumCode::Num2, false) | (NumCode::Num2, true) => '2',
            (NumCode::Num3, false) | (NumCode::Num3, true) => '3',
            (NumCode::Num4, false) | (NumCode::Num4, true) => '4',
            (NumCode::Num5, false) | (NumCode::Num5, true) => '5',
            (NumCode::Num6, false) | (NumCode::Num6, true) => '6',
            (NumCode::Num7, false) | (NumCode::Num7, true) => '7',
            (NumCode::Num8, false) | (NumCode::Num8, true) => '8',
            (NumCode::Num9, false) | (NumCode::Num9, true) => '9',
            (NumCode::A, false) => 'a',
            (NumCode::B, false) => 'b',
            (NumCode::C, false) => 'c',
            (NumCode::D, false) => 'd',
            (NumCode::E, false) => 'e',
            (NumCode::F, false) => 'f',
            (NumCode::G, false) => 'g',
            (NumCode::H, false) => 'h',
            (NumCode::I, false) => 'i',
            (NumCode::J, false) => 'j',
            (NumCode::K, false) => 'k',
            (NumCode::L, false) => 'l',
            (NumCode::M, false) => 'm',
            (NumCode::N, false) => 'n',
            (NumCode::O, false) => 'o',
            (NumCode::P, false) => 'p',
            (NumCode::Q, false) => 'q',
            (NumCode::R, false) => 'r',
            (NumCode::S, false) => 's',
            (NumCode::T, false) => 't',
            (NumCode::U, false) => 'u',
            (NumCode::V, false) => 'v',
            (NumCode::W, false) => 'w',
            (NumCode::X, false) => 'x',
            (NumCode::Y, false) => 'y',
            (NumCode::Z, false) => 'z',
            (NumCode::A, true) => 'A',
            (NumCode::B, true) => 'B',
            (NumCode::C, true) => 'C',
            (NumCode::D, true) => 'D',
            (NumCode::E, true) => 'E',
            (NumCode::F, true) => 'F',
            (NumCode::G, true) => 'G',
            (NumCode::H, true) => 'H',
            (NumCode::I, true) => 'I',
            (NumCode::J, true) => 'J',
            (NumCode::K, true) => 'K',
            (NumCode::L, true) => 'L',
            (NumCode::M, true) => 'M',
            (NumCode::N, true) => 'N',
            (NumCode::O, true) => 'O',
            (NumCode::P, true) => 'P',
            (NumCode::Q, true) => 'Q',
            (NumCode::R, true) => 'R',
            (NumCode::S, true) => 'S',
            (NumCode::T, true) => 'T',
            (NumCode::U, true) => 'U',
            (NumCode::V, true) => 'V',
            (NumCode::W, true) => 'W',
            (NumCode::X, true) => 'X',
            (NumCode::Y, true) => 'Y',
            (NumCode::Z, true) => 'Z',
            _ => '\0',
        };
        if c != '\0' {
            self.last_ascii = c;
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum NumCode {
    None = 0x0,
    Bad = 0xff,
    Return = 0x1c,
    Escape = 0x01,
    Space = 0x39,
    BackSpace = 0x0e,
    Tab = 0x0f,
    Alt = 0x38,
    Control = 0x1d,
    CapsLock = 0x3a,
    LShift = 0x2a,
    RShift = 0x36,
    UpArrow = 0x48,
    DownArrow = 0x50,
    LeftArrow = 0x4b,
    RightArrow = 0x4d,
    Insert = 0x52,
    Delete = 0x53,
    Home = 0x47,
    End = 0x4f,
    PgUp = 0x49,
    PgDn = 0x51,
    F1 = 0x3b,
    F2 = 0x3c,
    F3 = 0x3d,
    F4 = 0x3e,
    F5 = 0x3f,
    F6 = 0x40,
    F7 = 0x41,
    F8 = 0x42,
    F9 = 0x43,
    F10 = 0x44,
    F11 = 0x57,
    F12 = 0x59,
    Num1 = 0x02,
    Num2 = 0x03,
    Num3 = 0x04,
    Num4 = 0x05,
    Num5 = 0x06,
    Num6 = 0x07,
    Num7 = 0x08,
    Num8 = 0x09,
    Num9 = 0x0a,
    Num0 = 0x0b,
    A = 0x1e,
    B = 0x30,
    C = 0x2e,
    D = 0x20,
    E = 0x12,
    F = 0x21,
    G = 0x22,
    H = 0x23,
    I = 0x17,
    J = 0x24,
    K = 0x25,
    L = 0x26,
    M = 0x32,
    N = 0x31,
    O = 0x18,
    P = 0x19,
    Q = 0x10,
    R = 0x13,
    S = 0x1f,
    T = 0x14,
    U = 0x16,
    V = 0x2f,
    W = 0x11,
    X = 0x2d,
    Y = 0x15,
    Z = 0x2c,
}

pub fn to_numcode(v: u8) -> NumCode {
    match v {
        0x0 => NumCode::None,
        0xff => NumCode::Bad,
        0x1c => NumCode::Return,
        0x01 => NumCode::Escape,
        0x39 => NumCode::Space,
        0x0e => NumCode::BackSpace,
        0x0f => NumCode::Tab,
        0x38 => NumCode::Alt,
        0x1d => NumCode::Control,
        0x3a => NumCode::CapsLock,
        0x2a => NumCode::LShift,
        0x36 => NumCode::RShift,
        0x48 => NumCode::UpArrow,
        0x50 => NumCode::DownArrow,
        0x4b => NumCode::LeftArrow,
        0x4d => NumCode::RightArrow,
        0x52 => NumCode::Insert,
        0x53 => NumCode::Delete,
        0x47 => NumCode::Home,
        0x4f => NumCode::End,
        0x49 => NumCode::PgUp,
        0x51 => NumCode::PgDn,
        0x3b => NumCode::F1,
        0x3c => NumCode::F2,
        0x3d => NumCode::F3,
        0x3e => NumCode::F4,
        0x3f => NumCode::F5,
        0x40 => NumCode::F6,
        0x41 => NumCode::F7,
        0x42 => NumCode::F8,
        0x43 => NumCode::F9,
        0x44 => NumCode::F10,
        0x57 => NumCode::F11,
        0x59 => NumCode::F12,
        0x02 => NumCode::Num1,
        0x03 => NumCode::Num2,
        0x04 => NumCode::Num3,
        0x05 => NumCode::Num4,
        0x06 => NumCode::Num5,
        0x07 => NumCode::Num6,
        0x08 => NumCode::Num7,
        0x09 => NumCode::Num8,
        0x0a => NumCode::Num9,
        0x0b => NumCode::Num0,
        0x1e => NumCode::A,
        0x30 => NumCode::B,
        0x2e => NumCode::C,
        0x20 => NumCode::D,
        0x12 => NumCode::E,
        0x21 => NumCode::F,
        0x22 => NumCode::G,
        0x23 => NumCode::H,
        0x17 => NumCode::I,
        0x24 => NumCode::J,
        0x25 => NumCode::K,
        0x26 => NumCode::L,
        0x32 => NumCode::M,
        0x31 => NumCode::N,
        0x18 => NumCode::O,
        0x19 => NumCode::P,
        0x10 => NumCode::Q,
        0x13 => NumCode::R,
        0x1f => NumCode::S,
        0x14 => NumCode::T,
        0x16 => NumCode::U,
        0x2f => NumCode::V,
        0x11 => NumCode::W,
        0x2d => NumCode::X,
        0x15 => NumCode::Y,
        0x2c => NumCode::Z,
        _ => NumCode::None,
    }
}

pub struct Mouse {
    // TODO Mouse buttons
}

pub enum MouseCode {
    Left = 0x0,
    Right = 0x01,
    Middle = 0x02,
    //TODO define MouseButtons + delta?
}
