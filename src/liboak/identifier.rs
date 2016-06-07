// Copyright 2014 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub use std::string::String;
pub use rust::{Ident, Name, Span, str_lit};
use rust;
use std::ops::Deref;

pub fn id_to_string(id: Ident) -> String {
  id.to_string()
}

pub fn cook_lit(name: Name) -> String {
  str_lit(name.to_string().as_str())
}

pub fn string_to_lowercase(s: &String) -> String {
  s.chars().flat_map(char::to_lowercase).collect()
}

pub fn ident_to_lowercase(ident: Ident) -> String {
  let ident = id_to_string(ident);
  string_to_lowercase(&ident)
}

pub trait ItemIdent
{
  fn ident(&self) -> Ident;
}

pub trait ItemSpan
{
  fn span(&self) -> Span;
}

impl ItemIdent for rust::Item
{
  fn ident(&self) -> Ident {
    self.ident.clone()
  }
}

impl ItemSpan for rust::Item
{
  fn span(&self) -> Span {
    self.span.clone()
  }
}

impl<InnerItem: ItemIdent> ItemIdent for rust::P<InnerItem>
{
  fn ident(&self) -> Ident {
    self.deref().ident()
  }
}

impl<InnerItem: ItemSpan> ItemSpan for rust::P<InnerItem>
{
  fn span(&self) -> Span {
    self.deref().span()
  }
}
