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

pub fn ident_to_string(ident: Ident) -> String {
  ident.to_string()
}

pub fn string_to_ident(cx: &rust::ExtCtxt, name: String) -> Ident {
  cx.ident_of(name.as_str())
}

pub fn cook_lit(name: Name) -> String {
  str_lit(name.to_string().as_str(), None)
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
