use std::sync::{Arc, Mutex};

const NUM_KEYS : usize = 128;

#[derive(Clone)]
pub struct InputMonitoring {
    pub keyboard: Arc<Mutex<Keyboard>>,
    pub mouse: Arc<Mutex<Mouse>>,
}

impl InputMonitoring {
    pub fn clear_keyboard(&self) {
        let kb = &mut *self.keyboard.lock().unwrap();
        for i in 0..NUM_KEYS {
            kb.buttons[i] = false;
        }
    }

    pub fn key_pressed(&self) -> bool {
        let kb = &mut *self.keyboard.lock().unwrap();
        for i in 0..NUM_KEYS {
            if kb.buttons[i] == true {
                return true;
            }
        }
        false
    }
}

pub fn new_input_monitoring() -> InputMonitoring {
    let keyboard = Keyboard{
        buttons: [false; NUM_KEYS]
    };
    let mouse = Mouse {};
    
    InputMonitoring {
        keyboard: Arc::new(Mutex::new(keyboard)),
        mouse: Arc::new(Mutex::new(mouse)),
    }
}

pub struct Keyboard {
    pub buttons: [bool; NUM_KEYS],
}

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

pub struct Mouse {
    // TODO Mouse buttons
}

pub enum MouseCode {
    Left = 0x0,
    Right = 0x01,
    Middle = 0x02,
    //TODO define MouseButtons + delta?
}