//! A CSS parser that supports a tiny subset of CSS.

/// Parse a whole CSS stylesheet.
pub fn parse(source: String) -> Stylesheet {
    let mut parser = Parser {
        pos: 0,
        input: source,
    };
    parser.parse_rules()
}

#[derive(Debug, PartialEq)]
pub struct Stylesheet {
    rules: Vec<Rule>,
}

#[derive(Debug, PartialEq)]
struct Rule {
    /// Selectors are sorted, most-specific first.
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}

#[derive(Debug, PartialEq)]
enum Selector {
    /// Simple selectors.
    /// spec: https://www.w3.org/TR/CSS2/selector.html#selector-syntax
    Simple(SimpleSelector),
}

#[derive(Debug, PartialEq)]
struct SimpleSelector {
    tag_name: Option<String>,
    id: Option<String>,
    class: Vec<String>,
}

#[derive(Debug, PartialEq)]
struct Declaration {
    name: String,
    value: Value,
}

#[derive(Debug, PartialEq)]
enum Value {
    Keyword(String),
    Length(u32, Unit),
    Colorvalue(Color),
}

#[derive(Debug, PartialEq)]
enum Unit {
    Px,
}

#[derive(Debug, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

/// spec: https://www.w3.org/TR/selectors/#specificity
pub type Specificity = (usize, usize, usize);

impl Selector {
    /// spec: https://www.w3.org/TR/selectors/#specificity
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a, b, c)
    }
}

struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    fn parse_rules(&mut self) -> Stylesheet {
        let mut rules = vec![];
        loop {
            self.consume_whitespace();
            if self.eof() {
                break;
            }
            rules.push(self.parse_rule());
        }
        Stylesheet { rules }
    }

    /// Parse a CSS rule set.
    ///
    /// <rule> := <selectors> "{" <declarations> "}"
    fn parse_rule(&mut self) -> Rule {
        let selectors = self.parse_selectors();
        self.consume_char(); // "{"
        self.consume_whitespace();

        let declarations = self.parse_declarations();
        self.consume_char(); // "}"
        self.consume_whitespace();

        Rule {
            selectors,
            declarations,
        }
    }

    /// Parse a comma-separated list of selectors. Returned list is sorted by
    /// specificity.
    ///
    /// <selectors> := <selector> ("," <selector>)*
    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => {
                    // Start of declarations.
                    break;
                }
                c => panic!("Unexpected character {} in selector list", c),
            }
        }
        // Sort by specificities.
        selectors.sort_by(|a, b| b.specificity().cmp(&a.specificity()));
        selectors
    }

    /// Parse a simple selector, e.g., `type#id.class1.class2`.
    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: vec![],
        };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                'a'..='z' | 'A'..='Z' => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }
        selector
    }

    /// Parse a list of declarations.
    ///
    /// <declarations> := <decralation>*
    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut decls = vec![];
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                break;
            }
            decls.push(self.parse_declaration());
        }
        decls
    }

    /// Parse a declaration.
    ///
    /// <decralation> := ident ":" <value> ";"
    /// <value>       := <color> | <length> | ident
    fn parse_declaration(&mut self) -> Declaration {
        let name = self.parse_identifier();
        match self.next_char() {
            ':' => self.consume_char(),
            c => panic!("Expected a colon, but got {}.", c),
        };
        self.consume_whitespace();

        let value = match self.next_char() {
            '#' => self.parse_color(),
            '0'..='9' => self.parse_length(),
            _ => Value::Keyword(self.parse_identifier()),
        };
        match self.next_char() {
            ';' => self.consume_char(),
            c => panic!("Expected a semicolon, but got {}.", c),
        };

        Declaration { name, value }
    }

    /// Parse a color, e.g. `#aa2233`.
    fn parse_color(&mut self) -> Value {
        match self.next_char() {
            '#' => self.consume_char(),
            c => panic!("Expected #, but got {}.", c),
        };
        let (r, g, b) = (
            u8::from_str_radix(&self.input[self.pos..(self.pos + 2)], 16).unwrap(),
            u8::from_str_radix(&self.input[(self.pos + 2)..(self.pos + 4)], 16).unwrap(),
            u8::from_str_radix(&self.input[(self.pos + 4)..(self.pos + 6)], 16).unwrap(),
        );
        self.pos += 6;
        Value::Colorvalue(Color { r, g, b })
    }

    /// Parse a size, e.g. `24px`.
    fn parse_length(&mut self) -> Value {
        let num = {
            let mut num = "".to_string();
            loop {
                let c = self.next_char();
                match c {
                    '0'..='9' => {
                        num.push(c);
                        self.consume_char();
                    }
                    _ => break,
                };
            }
            u32::from_str_radix(&num, 10).unwrap()
        };

        let unit = {
            if self.input[self.pos..].starts_with("px") {
                self.pos += 2;
                Unit::Px
            } else {
                panic!("Unexpected unit, rather than \"px\".");
            }
        };

        Value::Length(num, unit)
    }

    fn parse_identifier(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => true,
            _ => false,
        })
    }

    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    fn consume_char(&mut self) -> char {
        let c = self.next_char();
        self.pos += c.len_utf8();
        c
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Consume and discard zero or more whitespace characters.
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_css_stylesheet() {
        let input = r#"
            h1, h2, h3 { margin: auto; color: #cc0000; }
            div.note { margin-bottom: 20px; padding: 10px; }
            #answer { display: none; }
        "#
        .to_string();
        let stylesheet = parse(input);

        let expected_rules = vec![
            Rule {
                selectors: vec![
                    Selector::Simple(SimpleSelector {
                        tag_name: Some("h1".to_string()),
                        id: None,
                        class: vec![],
                    }),
                    Selector::Simple(SimpleSelector {
                        tag_name: Some("h2".to_string()),
                        id: None,
                        class: vec![],
                    }),
                    Selector::Simple(SimpleSelector {
                        tag_name: Some("h3".to_string()),
                        id: None,
                        class: vec![],
                    }),
                ],
                declarations: vec![
                    Declaration {
                        name: "margin".to_string(),
                        value: Value::Keyword("auto".to_string()),
                    },
                    Declaration {
                        name: "color".to_string(),
                        value: Value::Colorvalue(Color { r: 204, g: 0, b: 0 }),
                    },
                ],
            },
            Rule {
                selectors: vec![Selector::Simple(SimpleSelector {
                    tag_name: Some("div".to_string()),
                    id: None,
                    class: vec!["note".to_string()],
                })],
                declarations: vec![
                    Declaration {
                        name: "margin-bottom".to_string(),
                        value: Value::Length(20, Unit::Px),
                    },
                    Declaration {
                        name: "padding".to_string(),
                        value: Value::Length(10, Unit::Px),
                    },
                ],
            },
            Rule {
                selectors: vec![Selector::Simple(SimpleSelector {
                    tag_name: None,
                    id: Some("answer".to_string()),
                    class: vec![],
                })],
                declarations: vec![Declaration {
                    name: "display".to_string(),
                    value: Value::Keyword("none".to_string()),
                }],
            },
        ];

        assert_eq!(
            stylesheet,
            Stylesheet {
                rules: expected_rules
            },
        );
    }
}

#[cfg(test)]
mod selector_tests {
    use super::*;

    #[test]
    fn specificity() {
        let selector = Selector::Simple(SimpleSelector {
            tag_name: None,
            id: Some("main".to_string()),
            class: vec![],
        });
        assert_eq!(selector.specificity(), (1, 0, 0));

        let selector = Selector::Simple(SimpleSelector {
            tag_name: Some("div".to_string()),
            id: Some("main".to_string()),
            class: vec!["someclass1".to_string(), "someclass2".to_string()],
        });
        assert_eq!(selector.specificity(), (1, 2, 1));
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn parse_rule() {
        let mut parser = Parser {
            pos: 0,
            input: r#"div#main.class1.class2 { margin: auto; display: block; }"#.to_string(),
        };
        let rule = parser.parse_rule();
        assert_eq!(
            rule,
            Rule {
                selectors: vec![Selector::Simple(SimpleSelector {
                    tag_name: Some("div".to_string()),
                    id: Some("main".to_string()),
                    class: vec!["class1".to_string(), "class2".to_string()],
                })],
                declarations: vec![
                    Declaration {
                        name: "margin".to_string(),
                        value: Value::Keyword("auto".to_string()),
                    },
                    Declaration {
                        name: "display".to_string(),
                        value: Value::Keyword("block".to_string()),
                    },
                ],
            },
        );
    }

    #[test]
    fn parse_selectors() {
        let mut parser = Parser {
            pos: 0,
            input: r#"h1, h2, div.class1.class2, p#id { ..."#.to_string(),
        };
        let selectors = parser.parse_selectors();
        assert_eq!(
            selectors,
            vec![
                Selector::Simple(SimpleSelector {
                    tag_name: Some("p".to_string()),
                    id: Some("id".to_string()),
                    class: vec![],
                }),
                Selector::Simple(SimpleSelector {
                    tag_name: Some("div".to_string()),
                    id: None,
                    class: vec!["class1".to_string(), "class2".to_string()],
                }),
                Selector::Simple(SimpleSelector {
                    tag_name: Some("h1".to_string()),
                    id: None,
                    class: vec![],
                }),
                Selector::Simple(SimpleSelector {
                    tag_name: Some("h2".to_string()),
                    id: None,
                    class: vec![],
                }),
            ]
        );
        assert_eq!(parser.pos, 32);
    }

    #[test]
    fn parse_simple_selector() {
        // id only
        let mut parser = Parser {
            pos: 0,
            input: r#"#id"#.to_string(),
        };
        assert_eq!(
            parser.parse_simple_selector(),
            SimpleSelector {
                tag_name: None,
                id: Some("id".to_string()),
                class: vec![],
            },
        );

        // classes only
        let mut parser = Parser {
            pos: 0,
            input: r#".class1.class2"#.to_string(),
        };
        assert_eq!(
            parser.parse_simple_selector(),
            SimpleSelector {
                tag_name: None,
                id: None,
                class: vec!["class1".to_string(), "class2".to_string()],
            },
        );

        // id + classes
        let mut parser = Parser {
            pos: 0,
            input: r#"#id.class1.class2"#.to_string(),
        };
        assert_eq!(
            parser.parse_simple_selector(),
            SimpleSelector {
                tag_name: None,
                id: Some("id".to_string()),
                class: vec!["class1".to_string(), "class2".to_string()],
            },
        );

        // tag name only
        let mut parser = Parser {
            pos: 0,
            input: r#"div"#.to_string(),
        };
        assert_eq!(
            parser.parse_simple_selector(),
            SimpleSelector {
                tag_name: Some("div".to_string()),
                id: None,
                class: vec![],
            },
        );

        // tag name + id + classes
        let mut parser = Parser {
            pos: 0,
            input: r#"div#id.class1.class2"#.to_string(),
        };
        assert_eq!(
            parser.parse_simple_selector(),
            SimpleSelector {
                tag_name: Some("div".to_string()),
                id: Some("id".to_string()),
                class: vec!["class1".to_string(), "class2".to_string()],
            },
        );
    }

    #[test]
    fn parse_declarations() {
        let mut parser = Parser {
            pos: 0,
            input: "margin: auto; display: block; } ...".to_string(),
        };
        let decls = parser.parse_declarations();
        assert_eq!(
            decls,
            vec![
                Declaration {
                    name: "margin".to_string(),
                    value: Value::Keyword("auto".to_string()),
                },
                Declaration {
                    name: "display".to_string(),
                    value: Value::Keyword("block".to_string()),
                },
            ],
        );
    }

    #[test]
    fn parse_declaration() {
        let mut parser = Parser {
            pos: 0,
            input: "margin: auto; ...".to_string(),
        };
        let decl = parser.parse_declaration();
        assert_eq!(
            decl,
            Declaration {
                name: "margin".to_string(),
                value: Value::Keyword("auto".to_string()),
            },
        );
    }

    #[test]
    fn parse_color() {
        let mut parser = Parser {
            pos: 0,
            input: "#aacc11;".to_string(),
        };
        let color = parser.parse_color();
        assert_eq!(
            color,
            Value::Colorvalue(Color {
                r: 170,
                g: 204,
                b: 17
            }),
        );
        assert_eq!(parser.pos, 7);
    }

    #[test]
    fn parse_length() {
        let mut parser = Parser {
            pos: 0,
            input: "123px;".to_string(),
        };
        let length = parser.parse_length();
        assert_eq!(length, Value::Length(123, Unit::Px));
        assert_eq!(parser.pos, 5);
    }

    #[test]
    fn parse_identifier() {
        let mut parser = Parser {
            pos: 0,
            input: "abc_ef...".to_string(),
        };
        assert_eq!(parser.parse_identifier(), "abc_ef".to_string());
    }

    #[test]
    fn next_char() {
        let mut parser = Parser {
            pos: 0,
            input: "abc".to_string(),
        };
        assert_eq!(parser.next_char(), 'a');

        parser.pos = 1;
        assert_eq!(parser.next_char(), 'b');
    }

    #[test]
    fn consume_char() {
        let mut parser = Parser {
            pos: 0,
            input: "abc".to_string(),
        };
        assert_eq!(parser.consume_char(), 'a');
        assert_eq!(parser.pos, 1);
        assert_eq!(parser.consume_char(), 'b');
        assert_eq!(parser.pos, 2);

        let mut parser = Parser {
            pos: 0,
            input: "あいう".to_string(),
        };
        assert_eq!(parser.consume_char(), 'あ');
        assert_eq!(parser.pos, 3);
        assert_eq!(parser.consume_char(), 'い');
        assert_eq!(parser.pos, 6);
    }

    #[test]
    fn eof() {
        let mut parser = Parser {
            pos: 2,
            input: "abc".to_string(),
        };
        assert!(!parser.eof());

        parser.pos = 3;
        assert!(parser.eof());
    }
}
