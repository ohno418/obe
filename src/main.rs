mod css;
mod dom;
mod html;
mod layout;
mod painting;
mod style;

fn main() {
    let html = r#"
        <div class="a">
          <div class="b">
            <div class="c">
              <div class="d">
                <div class="e">
                  <div class="f">
                    <div class="g">
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        "#;
    let css = r#"
        * { display: block; padding: 12px; }
        .a { background: #ff0000; }
        .b { background: #ffa500; }
        .c { background: #ffff00; }
        .d { background: #008000; }
        .e { background: #0000ff; }
        .f { background: #4b0082; }
        .g { background: #800080; }
        "#;

    let dom = html::parse(html.to_string());
    let cssom = css::parse(css.to_string());
    let style = style::style_tree(&dom, &cssom);

    // Since we don't have an actual window, hard-code the "viewport" size.
    let (width, height) = (800.0, 600.0);
    let mut viewport: layout::Dimensions = Default::default();
    viewport.content.width = width;
    viewport.content.height = height;

    let layout = layout::layout_tree(&style, &mut viewport);
    let canvas = painting::paint(&layout, width as usize, height as usize);

    // Save output as an image file.
    // let file = File::create(&Path::new("output.png")).unwrap();
    let (w, h) = (canvas.width as u32, canvas.height as u32);
    let buffer: Vec<image::Rgb<u8>> = unsafe { std::mem::transmute(canvas.pixels) };
    let img = image::ImageBuffer::from_fn(w, h, |x, y| buffer[(y * w + x) as usize]);
    image::DynamicImage::ImageRgb8(img)
        .save("output.png")
        .unwrap();
}
