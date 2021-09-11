use crate::{scanner::Scanner, token::TokenKind};

pub fn compile(source: &str) {
    let mut scanner = Scanner::new(source);
    let mut line = usize::MAX;
    loop {
        let token = scanner.next_token();
        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }

        println!("{:2} '{}'", token.kind, token.lexeme);

        if token.kind == TokenKind::Eof {
            break;
        }
    }
}
