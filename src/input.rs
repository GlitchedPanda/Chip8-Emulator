use winit::event::ElementState;
use winit::keyboard::KeyCode;

pub struct Input {
    key_map: [bool; 16], 
}

impl Input {
    pub fn new() -> Self {
        Input {
            key_map: [false; 16],
        }
    }

    // Update the key map with the latest input state
    pub fn update_key_state(&mut self, key: KeyCode, state: ElementState) {
        let key_index = match key {
            KeyCode::Digit1 => Some(0x1),
            KeyCode::Digit2 => Some(0x2),
            KeyCode::Digit3 => Some(0x3),
            KeyCode::Digit4 => Some(0xc),
            KeyCode::KeyQ => Some(0x4),
            KeyCode::KeyW => Some(0x5),
            KeyCode::KeyE => Some(0x6),
            KeyCode::KeyR => Some(0xd),
            KeyCode::KeyA => Some(7),
            KeyCode::KeyS => Some(8),
            KeyCode::KeyD => Some(9),
            KeyCode::KeyF => Some(0xe),
            KeyCode::KeyZ => Some(0xa),
            KeyCode::KeyX => Some(0x0),
            KeyCode::KeyC => Some(0xb),
            KeyCode::KeyV => Some(0xf),
            _ => None,
        };

        if let Some(index) = key_index {
            self.key_map[index] = state == ElementState::Pressed;
        }
    }

    pub fn get_key_map(&self) -> &[bool; 16] {
        &self.key_map
    }
}

