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
    let mut viewport: layout::Dimensions = Default::default();
    let layout = layout::layout_tree(&style, &mut viewport);
    let canvas = painting::paint(&layout, 1024, 1024);

    dbg!(canvas);
}
