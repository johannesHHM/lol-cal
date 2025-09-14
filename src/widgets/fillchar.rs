use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

#[derive(Clone, Copy, Debug)]
pub struct FillChar {
    ch: char,
    style: Style,
}

impl FillChar {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            style: Style::default(),
        }
    }

    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
}

impl Widget for FillChar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let fill_line: String = std::iter::repeat(self.ch)
            .take(area.width as usize)
            .collect();

        for y in area.y..area.y + area.height {
            buf.set_string(area.x, y, &fill_line, self.style);
        }
    }
}
