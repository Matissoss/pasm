pub mod parser;
mod tokenizer;
mod lexer;
mod label;
mod instruction;
mod register;

pub trait FromToken{
    fn from_token(tokens: &[tokenizer::Token]) -> Option<Self>
        where Self: Sized;
}
