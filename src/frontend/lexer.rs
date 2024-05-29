use logos::Logos;

#[derive(Debug, PartialEq, Clone, Default)]
enum LexingError {
    NumberParseError,
    #[default]
    Other,
}

impl From<std::num::ParseIntError> for LexingError {
    fn from(_: std::num::ParseIntError) -> Self {
        LexingError::NumberParseError
    }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
#[logos(error = LexingError)]
enum Token {
    #[token("fn")]
    Fn,
    #[token("return")]
    Return,
    #[token("let")]
    Let,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("loop")]
    Loop,
    #[token("@")]
    At,
    #[token("break")]
    Break,
    #[token("(")]
    LPar,
    #[token(")")]
    RPar,
    #[token(";")]
    Semicol,
    #[token("{")]
    LCurl,
    #[token("}")]
    RCurl,
    #[token(":")]
    Colon,
    #[token("+")]
    Plus,
    #[token("*")]
    Mult,
    #[token("-")]
    Minus,
    #[token("==")]
    Eq,
    #[token("=")]
    Assign,
    #[token("<")]
    Smaller,
    #[token(">")]
    Larger,
    #[token("!")]
    Not,
    #[token("!=")]
    NotEq,
    #[token("..")]
    DotDot,
    #[regex("[a-zA-Z][a-zA-Z_0-9]*", |lex| lex.slice().to_string())]
    Id(String),
    #[regex("-?[0-9]+", |lex| lex.slice().parse())]
    Num(i64),
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    Bool(bool),
}

#[cfg(test)]
mod lexertests {
    use super::*;

    #[test]
    fn shoudld_parse() {
        let result = Token::lexer("fn something() {}").collect::<Vec<_>>();
        assert_eq!(
            result,
            &[
                Ok(Token::Fn),
                Ok(Token::Id(String::from("something"))),
                Ok(Token::LPar),
                Ok(Token::RPar),
                Ok(Token::LCurl),
                Ok(Token::RCurl)
            ]
        )
    }
}
