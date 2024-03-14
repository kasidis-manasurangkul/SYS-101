pub struct Player {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub color: (u8, u8, u8),
}

impl Player {
    pub fn new(x: usize, y: usize, width: usize, height: usize, color: (u8, u8, u8)) -> Self {
        Player {
            x,
            y,
            width,
            height,
            color,
        }
    }

    pub fn draw(&self, writer: &mut ScreenWriter) {
        for dx in 0..self.width {
            for dy in 0..self.height {
                writer.draw_pixel(
                    self.x + dx,
                    self.y + dy,
                    self.color.0,
                    self.color.1,
                    self.color.2,
                );
            }
        }
    }
}