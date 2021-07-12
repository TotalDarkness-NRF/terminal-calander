use crate::terminal::Terminal;

#[derive(Clone, Copy)]
pub struct Position {
    x: u16,
    y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Position { x, y }
    }

    pub fn new_origin() -> Self {
        Position::new(1, 2)
    }

    pub fn get_x(&self) -> u16 {
        self.x
    }

    pub fn get_y(&self) -> u16 {
        self.y
    }

    pub fn move_up(&mut self) -> bool {
        self.set_y(self.y - 1)
    }

    pub fn move_down(&mut self) -> bool {
        self.set_y(self.y + 1)
    }

    pub fn move_right(&mut self) -> bool {
        self.set_x(self.x + 1)
    }

    pub fn move_left(&mut self) -> bool {
        self.set_x(self.x - 1)
    }

    pub fn set_x(&mut self, x: u16) -> bool {
        self.set(x, self.y)
    }

    pub fn set_y(&mut self, y: u16) -> bool {
        self.set(self.x, y)
    }

    pub fn set(&mut self, x: u16, y: u16) -> bool {
        if self.respect_boundary(x, y) {
            self.x = x;
            self.y = y;
            return true;
        }
        false
    }

    fn respect_boundary(&self, x: u16, y: u16) -> bool {
        // Check current and future positions
        self.is_in_boundary() && x > 0 && y > 1 && Position::new(x, y).is_in_boundary()
    }

    pub fn is_in_boundary(&self) -> bool {
        self.x > 0 && self.y > 0 && {
            let boundary = Terminal::get_boundaries();
            self.x <= boundary.x && self.y <= boundary.y
        }
    }
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}