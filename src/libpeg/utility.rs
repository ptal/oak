pub use std::string::String;
pub use syntax::ast::{Ident, Name};

use syntax::parse::token;

pub fn id_to_string(id: Ident) -> String
{
  String::from_str(token::get_ident(id).get())
}

pub fn name_to_string(name: Name) -> String
{
  String::from_str(token::get_name(name).get())
}