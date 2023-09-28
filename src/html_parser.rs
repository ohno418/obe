use crate::dom;

/// Parse an HTML document and return the root element.
pub fn parse(source: String) -> dom::Node {
    let nodes = Parser {
        pos: 0,
        input: source,
    }
    .parse_nodes();
    dom::elem("html".to_string(), dom::AttrMap::new(), nodes)
}

#[derive(Debug)]
struct Parser {
    /// The index of the next character that hasn't be processed yet.
    pos: usize,
    /// The whole input string.
    input: String,
}

impl Parser {
    /// Read the current character without consuming it.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    /// Return true if the next characters starts with the given string.
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    /// Return true if all input is consumed.
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Return the current character, and advance pos to the next character.
    fn consume_char(&mut self) -> char {
        let c = self.next_char();
        self.pos += c.len_utf8();
        c
    }

    /// Consume characters until `test` returns false.
    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut s = String::new();
        while !self.eof() && test(self.next_char()) {
            s.push(self.consume_char());
        }
        s
    }

    /// Consume and discard zero or more whitespace characters.
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    /// Parse a tag or attribute name.
    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => true,
            _ => false,
        })
    }

    /// Parse a single node.
    fn parse_node(&mut self) -> dom::Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        }
    }

    /// Parse a text node.
    fn parse_text(&mut self) -> dom::Node {
        dom::text(self.consume_while(|c| c != '<'))
    }

    /// Parse a single element, including its open tag, contents, and closing tag.
    fn parse_element(&mut self) -> dom::Node {
        // opening tag
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        assert!(self.consume_char() == '>');

        // contents
        let children = self.parse_nodes();

        // closing tag
        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        dom::elem(tag_name, attrs, children)
    }

    /// Parse a single name="value" pair.
    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        (name, value)
    }

    /// Parse a quoted value.
    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        self.consume_char();
        value
    }

    /// Parse a list of name="value" pairs, separated by whitespace.
    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attrs = dom::AttrMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attrs.insert(name, value);
        }
        attrs
    }

    /// Parse a sequence of sibling nodes.
    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_html_document() {
        let source = "<div><div id=\"main\">hello</div><p>parag</p></div>".to_string();
        assert_eq!(
            parse(source),
            dom::elem(
                String::from("html"),
                dom::AttrMap::new(),
                vec![dom::elem(
                    String::from("div"),
                    dom::AttrMap::new(),
                    vec![
                        dom::elem(
                            String::from("div"),
                            dom::AttrMap::from([("id".to_string(), "main".to_string()),]),
                            vec![dom::text("hello".to_string())],
                        ),
                        dom::elem(
                            String::from("p"),
                            dom::AttrMap::new(),
                            vec![dom::text("parag".to_string())],
                        ),
                    ],
                ),],
            ),
        );
    }

    #[test]
    fn next_char() {
        let mut parser = Parser {
            pos: 0,
            input: String::from("Hello, world!"),
        };
        assert_eq!(parser.next_char(), 'H');

        parser.pos = 4;
        assert_eq!(parser.next_char(), 'o');
    }

    #[test]
    fn starts_with() {
        let parser = Parser {
            pos: 2,
            input: String::from("Hello, world!"),
        };
        assert!(parser.starts_with("llo"));
        assert!(!parser.starts_with("lo"));
    }

    #[test]
    fn eof() {
        let mut parser = Parser {
            pos: 2,
            input: String::from("Hello, world!"),
        };
        assert!(!parser.eof());

        parser.pos = 13;
        assert!(parser.eof());

        // over
        parser.pos = 14;
        assert!(parser.eof());
    }

    #[test]
    fn consume_char() {
        let mut parser = Parser {
            pos: 0,
            input: String::from("Hello, world!"),
        };
        assert_eq!(parser.consume_char(), 'H');
        assert_eq!(parser.pos, 1);
        assert_eq!(parser.consume_char(), 'e');
        assert_eq!(parser.pos, 2);

        // including multi-byte character
        let mut parser = Parser {
            pos: 0,
            input: String::from("ハロー"),
        };
        assert_eq!(parser.consume_char(), 'ハ');
        assert_eq!(parser.pos, 3);
        assert_eq!(parser.consume_char(), 'ロ');
        assert_eq!(parser.pos, 6);
    }

    #[test]
    fn consume_while() {
        let mut parser = Parser {
            pos: 0,
            input: String::from("Hello, world!"),
        };
        let s = parser.consume_while(|c| c != ',');
        assert_eq!(s, String::from("Hello"));
        assert_eq!(parser.pos, 5);

        // till eof
        let s = parser.consume_while(|c| c != 'Z');
        assert_eq!(s, String::from(", world!"));
        assert_eq!(parser.pos, 13);
    }

    #[test]
    fn consume_whitespace() {
        let mut parser = Parser {
            pos: 0,
            input: String::from("!   John."),
        };

        // consume nothing
        parser.consume_whitespace();
        assert_eq!(parser.pos, 0);

        parser.pos += 1;

        parser.consume_whitespace();
        assert_eq!(parser.pos, 4);
    }

    #[test]
    fn parse_tag_name() {
        let mut parser = Parser {
            pos: 0,
            input: String::from("Hello, this is me."),
        };

        let s = parser.parse_tag_name();
        assert_eq!(s, String::from("Hello"));
    }

    #[test]
    fn parse_text() {
        let mut parser = Parser {
            pos: 5,
            input: String::from("<div>hello</div>"),
        };
        assert_eq!(parser.parse_text(), dom::text(String::from("hello")),);
    }

    #[test]
    fn parse_element() {
        let mut parser = Parser {
            pos: 0,
            input: String::from("<div>hello</div>"),
        };
        assert_eq!(
            parser.parse_element(),
            dom::elem(
                String::from("div"),
                dom::AttrMap::new(),
                vec![dom::text("hello".to_string())],
            ),
        );

        // nested
        let mut parser = Parser {
            pos: 0,
            input: String::from("<div><p>parag</p></div>"),
        };
        assert_eq!(
            parser.parse_element(),
            dom::elem(
                String::from("div"),
                dom::AttrMap::new(),
                vec![dom::elem(
                    String::from("p"),
                    dom::AttrMap::new(),
                    vec![dom::text("parag".to_string()),],
                ),],
            ),
        );

        // with attrs
        let mut parser = Parser {
            pos: 0,
            input: String::from("<div id=\"main\" class=\"test\">hello</div>"),
        };
        assert_eq!(
            parser.parse_element(),
            dom::elem(
                String::from("div"),
                dom::AttrMap::from([
                    ("id".to_string(), "main".to_string()),
                    ("class".to_string(), "test".to_string()),
                ]),
                vec![dom::text("hello".to_string())],
            ),
        );
    }

    #[test]
    fn parse_attributes() {
        let mut parser = Parser {
            pos: 0,
            input: String::from("id=\"main\" class=\"someclass\" >"),
        };
        assert_eq!(
            parser.parse_attributes(),
            dom::AttrMap::from([
                ("id".to_string(), "main".to_string()),
                ("class".to_string(), "someclass".to_string()),
            ]),
        );
    }

    #[test]
    fn parse_nodes() {
        let mut parser = Parser {
            pos: 0,
            input: String::from("<div id=\"main\">hello</div><p>parag</p>"),
        };
        assert_eq!(
            parser.parse_nodes(),
            vec![
                dom::elem(
                    String::from("div"),
                    dom::AttrMap::from([("id".to_string(), "main".to_string()),]),
                    vec![dom::text("hello".to_string())],
                ),
                dom::elem(
                    String::from("p"),
                    dom::AttrMap::new(),
                    vec![dom::text("parag".to_string())],
                ),
            ],
        );
    }
}
