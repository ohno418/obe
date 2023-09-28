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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
