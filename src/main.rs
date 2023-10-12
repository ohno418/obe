mod css_parser;
mod dom;
mod html_parser;
mod layout;
mod style;

fn main() {
    let dom = html_parser::parse(
        r#"
        <html>
            <body>
                <h1>Title</h1>
                <div id="main" class="test">
                    <p>Hello <em>world</em>!</p>
                </div>
            </body>
        </html>"#
            .to_string(),
    );
    let cssom = css_parser::parse(
        r#"
        h1, h2, h3 { margin: auto; color: #cc0000; }
        div.note { margin-bottom: 20px; padding: 10px; }
        #answer { display: none; }"#
            .to_string(),
    );
    let style = style::style_tree(&dom, &cssom);
    let mut viewport: layout::Dimensions = Default::default();
    let layout = layout::layout_tree(&style, &mut viewport);
    dbg!(layout);
}
