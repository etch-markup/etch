#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Command(String),
    Text(String),
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Superscript,
    Subscript,
    Whitespace(String),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut text = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                flush_text(&mut tokens, &mut text);

                let Some(next) = chars.peek().copied() else {
                    text.push('\\');
                    continue;
                };

                if next.is_ascii_alphabetic() {
                    let mut command = String::new();
                    while let Some(letter) = chars.peek().copied() {
                        if !letter.is_ascii_alphabetic() {
                            break;
                        }
                        command.push(letter);
                        chars.next();
                    }

                    match command.as_str() {
                        "quad" | "qquad" => tokens.push(Token::Whitespace(format!("\\{command}"))),
                        _ => tokens.push(Token::Command(command)),
                    }
                    continue;
                }

                let escaped = chars.next().expect("peeked escaped char should exist");
                match escaped {
                    ',' | ';' | '\\' => {
                        tokens.push(Token::Whitespace(format!("\\{escaped}")));
                    }
                    '{' | '}' | '[' | ']' | '^' | '_' => {
                        text.push(escaped);
                    }
                    _ => text.push(escaped),
                }
            }
            '{' => {
                flush_text(&mut tokens, &mut text);
                tokens.push(Token::OpenBrace);
            }
            '}' => {
                flush_text(&mut tokens, &mut text);
                tokens.push(Token::CloseBrace);
            }
            '[' => {
                flush_text(&mut tokens, &mut text);
                tokens.push(Token::OpenBracket);
            }
            ']' => {
                flush_text(&mut tokens, &mut text);
                tokens.push(Token::CloseBracket);
            }
            '^' => {
                flush_text(&mut tokens, &mut text);
                tokens.push(Token::Superscript);
            }
            '_' => {
                flush_text(&mut tokens, &mut text);
                tokens.push(Token::Subscript);
            }
            c if c.is_whitespace() => {
                flush_text(&mut tokens, &mut text);
                let mut whitespace = String::from(c);
                while let Some(next) = chars.peek().copied() {
                    if !next.is_whitespace() {
                        break;
                    }
                    whitespace.push(next);
                    chars.next();
                }
                tokens.push(Token::Whitespace(whitespace));
            }
            _ => text.push(ch),
        }
    }

    flush_text(&mut tokens, &mut text);
    tokens
}

fn flush_text(tokens: &mut Vec<Token>, text: &mut String) {
    if text.is_empty() {
        return;
    }

    tokens.push(Token::Text(std::mem::take(text)));
}

#[cfg(test)]
mod tests {
    use super::{Token, tokenize};

    #[test]
    fn tokenizes_simple_expression() {
        assert_eq!(
            tokenize("x^2 + y"),
            vec![
                Token::Text("x".into()),
                Token::Superscript,
                Token::Text("2".into()),
                Token::Whitespace(" ".into()),
                Token::Text("+".into()),
                Token::Whitespace(" ".into()),
                Token::Text("y".into()),
            ]
        );
    }

    #[test]
    fn tokenizes_commands_and_groups() {
        assert_eq!(
            tokenize("\\frac{1}{2}"),
            vec![
                Token::Command("frac".into()),
                Token::OpenBrace,
                Token::Text("1".into()),
                Token::CloseBrace,
                Token::OpenBrace,
                Token::Text("2".into()),
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn tokenizes_spacing_commands() {
        assert_eq!(
            tokenize("\\, \\; \\quad \\qquad"),
            vec![
                Token::Whitespace("\\,".into()),
                Token::Whitespace(" ".into()),
                Token::Whitespace("\\;".into()),
                Token::Whitespace(" ".into()),
                Token::Whitespace("\\quad".into()),
                Token::Whitespace(" ".into()),
                Token::Whitespace("\\qquad".into()),
            ]
        );
    }

    #[test]
    fn tokenizes_mixed_expression() {
        assert_eq!(
            tokenize("\\int_0^1 x^2 \\, dx"),
            vec![
                Token::Command("int".into()),
                Token::Subscript,
                Token::Text("0".into()),
                Token::Superscript,
                Token::Text("1".into()),
                Token::Whitespace(" ".into()),
                Token::Text("x".into()),
                Token::Superscript,
                Token::Text("2".into()),
                Token::Whitespace(" ".into()),
                Token::Whitespace("\\,".into()),
                Token::Whitespace(" ".into()),
                Token::Text("dx".into()),
            ]
        );
    }
}
