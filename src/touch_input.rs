use ctru::prelude::*;

const SWIPE_THRESHOLD: i32 = 30;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

impl SwipeDirection {
    pub fn as_char(self) -> char {
        match self {
            SwipeDirection::Up => 'u',
            SwipeDirection::Down => 'd',
            SwipeDirection::Left => 'l',
            SwipeDirection::Right => 'r',
        }
    }
}

pub struct TouchInputHandler {
    anchor: Option<(i32, i32)>,
}

impl TouchInputHandler {
    pub fn new() -> Self {
        Self { anchor: None }
    }

    pub fn update(&mut self, hid: &Hid) -> Option<SwipeDirection> {
        if !hid.keys_held().contains(KeyPad::TOUCH) {
            self.anchor = None;
            return None;
        }

        let (raw_x, raw_y) = hid.touch_position();
        let (x, y) = (raw_x as i32, raw_y as i32);

        let (anchor_x, anchor_y) = match self.anchor {
            Some(pos) => pos,
            None => {
                self.anchor = Some((x, y));
                return None;
            }
        };

        let dx = x - anchor_x;
        let dy = y - anchor_y;

        if dx.abs() < SWIPE_THRESHOLD && dy.abs() < SWIPE_THRESHOLD {
            return None;
        }

        let direction = if dx.abs() > dy.abs() {
            if dx > 0 { SwipeDirection::Right } else { SwipeDirection::Left }
        } else {
            if dy > 0 { SwipeDirection::Down } else { SwipeDirection::Up }
        };

        self.anchor = Some((x, y));

        Some(direction)
    }
}