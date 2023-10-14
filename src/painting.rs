use crate::css::{Color, Value};
use crate::layout::{BoxType, LayoutBox, Rect};

use std::fmt;

pub struct Canvas {
    pub pixels: Vec<Color>,
    pub width: usize,
    pub height: usize,
}

impl fmt::Debug for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Canvas width={} height={} pixels:\n", self.width, self.height)?;
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.pixels[x + y * self.width];
                write!(f, "#{:02x}{:02x}{:02x} ", pixel.r, pixel.g, pixel.b)?;
            }
            write!(f, "\n")?;
        }
        fmt::Result::Ok(())
    }
}

pub fn paint(layout_box: &LayoutBox, width: usize, height: usize) -> Canvas {
    let display_list = build_display_list(layout_box);
    let mut canvas = Canvas::new(width, height);
    for item in display_list {
        canvas.paint_item(&item);
    }
    canvas
}

/// Display list, a list of display commands to execute.
type DisplayList = Vec<DisplayCommand>;

#[derive(Debug)]
enum DisplayCommand {
    /// Paint a solid-color rectangle.
    SolidColor(Color, Rect),
}

fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = Vec::new();
    render_layout_box(&mut list, layout_root);
    list
}

fn render_layout_box(list: &mut DisplayList, layout_box: &LayoutBox) {
    render_background(list, layout_box);
    render_borders(list, layout_box);

    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}

fn render_background(list: &mut DisplayList, layout_box: &LayoutBox) {
    get_color(layout_box, "background").map(|color| {
        list.push(DisplayCommand::SolidColor(
            color,
            layout_box.dimensions.border_box(),
        ))
    });
}

fn render_borders(list: &mut DisplayList, layout_box: &LayoutBox) {
    let Some(color) = get_color(layout_box, "border-color") else {
        // Bail out if no border-color is specified.
        return;
    };

    let d = &layout_box.dimensions;
    let border_box = d.border_box();

    // Left border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y,
            width: d.border.left,
            height: border_box.height,
        },
    ));

    // Right border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x + border_box.width - d.border.right,
            y: border_box.y,
            width: d.border.right,
            height: border_box.height,
        },
    ));

    // Top border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y,
            width: border_box.width,
            height: d.border.top,
        },
    ));

    // Bottom border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y + border_box.height - d.border.bottom,
            width: border_box.width,
            height: d.border.bottom,
        },
    ));
}

/// Return the specified color for CSS property `name`, or None if no color was
/// specified.
fn get_color(layout_box: &LayoutBox, name: &str) -> Option<Color> {
    match layout_box.box_type {
        BoxType::BlockNode(style) | BoxType::InlineNode(style) => {
            if let Some(Value::Colorvalue(color)) = style.value(name) {
                Some(color)
            } else {
                None
            }
        }
        BoxType::AnonymousBlock => None,
    }
}

impl Canvas {
    /// Create a new blank canvas.
    fn new(width: usize, height: usize) -> Canvas {
        let white = Color {
            r: 255,
            g: 255,
            b: 255,
        };
        Canvas {
            pixels: vec![white; width * height],
            width,
            height,
        }
    }

    /// Execute a display command and paint into the canvas.
    fn paint_item(&mut self, item: &DisplayCommand) {
        match item {
            DisplayCommand::SolidColor(color, rect) => {
                let x0 = rect.x as usize;
                let x1 = x0 + rect.width as usize;
                let y0 = rect.y as usize;
                let y1 = y0 + rect.height as usize;

                for y in y0..y1 {
                    for x in x0..x1 {
                        self.pixels[x + y * self.width] = *color;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod canvas_tests {
    use super::*;

    #[test]
    fn new() {
        let canvas = Canvas::new(8, 8);

        assert_eq!(canvas.width, 8);
        assert_eq!(canvas.height, 8);

        let pixels = canvas.pixels;
        let white = Color {
            r: 255,
            g: 255,
            b: 255,
        };
        assert_eq!(pixels.len(), 64);
        assert!(pixels.iter().all(|p| *p == white));
    }

    #[test]
    fn paint_item() {
        let mut canvas = Canvas::new(4, 4);

        let black = Color { r: 0, g: 0, b: 0 };
        let rect = Rect {
            x: 2.0,
            y: 1.0,
            width: 2.0,
            height: 3.0,
        };
        let item = DisplayCommand::SolidColor(black, rect);

        canvas.paint_item(&item);

        assert_eq!(canvas.width, 4);
        assert_eq!(canvas.height, 4);

        let pixels = canvas.pixels;
        let white = Color {
            r: 255,
            g: 255,
            b: 255,
        };
        assert_eq!(pixels.len(), 16);
        assert_eq!(pixels[0..4], vec![white; 4]);
        assert_eq!(pixels[4..8], vec![white, white, black, black]);
        assert_eq!(pixels[8..12], vec![white, white, black, black]);
        assert_eq!(pixels[12..16], vec![white, white, black, black]);
    }
}
