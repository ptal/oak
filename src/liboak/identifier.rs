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
pub use proc_macro2::{Ident,Span};

pub trait ItemIdent
{
  fn ident(&self) -> Ident;
}

impl ItemIdent for syn::Item
{
  fn ident(&self) -> Ident {
    use syn::Item::*;
    match self {
      Const(item) => item.ident.clone(),
      Enum(item) => item.ident.clone(),
      ExternCrate(item) => item.ident.clone(),
      Fn(item) => item.sig.ident.clone(),
      ForeignMod(_) => panic!("[bug] `ForeignMod` has no identifier (please report this issue)."),
      Impl(_) => panic!("[bug] `Impl` has no identifier (please report this issue)."),
      Macro(_) => panic!("[bug] `Macro` has no identifier (please report this issue)."),
      Macro2(item) => item.ident.clone(),
      Mod(item) => item.ident.clone(),
      Static(item) => item.ident.clone(),
      Struct(item) => item.ident.clone(),
      Trait(item) => item.ident.clone(),
      TraitAlias(item) => item.ident.clone(),
      Type(item) => item.ident.clone(),
      Union(item) => item.ident.clone(),
      Use(_) => panic!("[bug] `Use` has no identifier (please report this issue)."),
      Verbatim(_) => panic!("[bug] `Verbatim` has no identifier (please report this issue)."),
      _ => panic!("[bug] non exhaustive case in ItemIdent for syn::Item (please report this issue)."),
    }
  }
}

impl ItemIdent for syn::ItemFn
{
  fn ident(&self) -> Ident { self.sig.ident.clone() }
}