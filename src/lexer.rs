mod token_type;
pub use token_type::*;
mod token_number;
use token_number::token_number;
mod token_string;
use token_string::token_string;
use util::{Position, Token};
mod token_identifier;

use crate::{
    internal::errors::{show_error, show_multiple_errors, ErrorNames, ErrorTypes},
    util::{split_meta, to_cyan},
};
use token_identifier::token_identifier;

const NUMBERS: &str = "0123456789";
const OPERATORS: &str = "+-*/%=&|<>!^~?";
const PUNCTUATION: &str = "(){}[],.;:";

fn token_error(token: &util::Token<TokenType>) -> ErrorTypes {
    let (file_name, data_line, token_value) = split_meta(&token.meta);
    let line = token.location.start.line + 1;
    let column_token = token.location.start.column + 1;
    let column = token.location.end.column;
    let str_line = line.to_string();
    let str_init = " ".repeat(str_line.len());

    let cyan_line = to_cyan("|");
    let cyan_arrow = to_cyan("-->");

    let indicator = if token_value.len() > 0 {
        format!("{}^", "-".repeat(token_value.len()))
    } else {
        "^".to_string()
    };
    let lines = [
        format!("{}", token.value),
        format!("{}{cyan_arrow} {}:{}:{}", str_init, file_name, line, column),
        format!("{} {cyan_line}", str_init),
        format!("{} {cyan_line} {}", to_cyan(&str_line), data_line),
        format!(
            "{} {cyan_line} {}{}",
            str_init,
            " ".repeat(column_token - 1),
            to_cyan(&indicator)
        ),
        format!("{} {cyan_line}", str_init),
    ];
    let joined = lines.join("\n");
    ErrorTypes::StringError(joined)
}

fn comment(_: char, position: Position, line:String, meta:String) -> (Token<TokenType>, usize){
    let line_len = line.len();
    let length = line_len - position.column;
    let token = Token {
        token_type: TokenType::None,
        value: "".to_string(),
        location: util::Location {
            start: position,
            end: Position { line: position.line, column:length },
            length
        },
        meta,
    };
    (token, length+1)
}

pub fn tokenizer(input: String, file_name: String) -> Vec<util::Token<TokenType>> {
    let tokens = util::tokenize::<TokenType>(
        input,
        vec![
            (
                util::TokenOptionCondition::Chars("\t\r "),
                util::TokenOptionResult::Min(|| TokenType::None),
            ),
            (
                util::TokenOptionCondition::Chars(NUMBERS),
                util::TokenOptionResult::Full(token_number),
            ),
            (
                util::TokenOptionCondition::Fn(|c| c.is_alphabetic() || "_$".contains(c)),
                util::TokenOptionResult::Full(token_identifier),
            ),
            (
                util::TokenOptionCondition::Chars(OPERATORS),
                util::TokenOptionResult::Char(|c| TokenType::Operator(OperatorType::from(c))),
            ),
            (
                util::TokenOptionCondition::Chars("'\""),
                util::TokenOptionResult::Full(token_string),
            ),
            (
                util::TokenOptionCondition::Chars(PUNCTUATION),
                util::TokenOptionResult::Char(|c| TokenType::Punctuation(PunctuationType::from(c))),
            ),
            (
                util::TokenOptionCondition::Chars("#"),
                util::TokenOptionResult::Full(comment),
            ),
        ],
        file_name,
    );
    let tokens = match tokens {
        Ok(mut t) => {
            let end_token = t.get(t.len() - 1).unwrap();
            let pos = util::Position {
                line: end_token.location.end.line,
                column: end_token.location.end.column + 1,
            };
            t.push(util::Token {
                token_type: TokenType::EOF,
                location: util::Location {
                    start: pos,
                    end: pos,
                    length: 0
                },
                value: "".to_string(),
                meta: "".to_string(),
            });
            t.retain(|x| x.token_type != TokenType::None);
            t
        }
        Err(e) => {
            show_error(&ErrorNames::LexerError, ErrorTypes::ErrorError(e));
            return Vec::new();
        }
    };
    let errors = tokens
        .iter()
        .filter(|x| x.token_type == TokenType::Error)
        .map(|x| token_error(x))
        .collect::<Vec<ErrorTypes>>();
    if errors.len() > 0 {
        show_multiple_errors(&ErrorNames::LexerError, errors);
        return Vec::new();
    }
    tokens
}
