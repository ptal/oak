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

extern crate unicode;

use rust;
pub use std::string::String;
pub use rust::{Ident, Name};

pub fn id_to_string(id: Ident) -> String
{
  String::from_str(rust::get_ident(id).get())
}

pub fn name_to_string(name: Name) -> String
{
  String::from_str(rust::get_name(name).get())
}

pub fn string_to_lowercase(s: &String) -> String
{
  let mut res = String::new();
  for c in s.as_slice().chars()
    .map(|c|c.to_lowercase())
  {
    res.push(c);
  }
  res
}

// Adapted from the Rust compiler source (librustc/lint/builtin.rs)
pub fn is_camel_case(ident: Ident) -> bool
{
  let ident = rust::get_ident(ident);
  assert!(!ident.get().is_empty());
  let ident = ident.get().trim_matches('_');

  // start with a non-lowercase letter rather than non-uppercase
  // ones (some scripts don't have a concept of upper/lowercase)
  !ident.char_at(0).is_lowercase() && !ident.contains_char('_')
}

// Adapted from the Rust compiler source (librustc/lint/builtin.rs)
pub fn to_camel_case(s: &str) -> String
{
  s.split('_').flat_map(|word| word.chars().enumerate().map(|(i, c)|
    if i == 0 { c.to_uppercase() }
    else { c }
  )).collect()
}

pub fn id_to_camel_case(ident: Ident) -> String
{
  if !is_camel_case(ident.clone()) {
    to_camel_case(rust::get_ident(ident).get())
  } else {
    id_to_string(ident)
  }
}

// Adapted from the Rust compiler source (librustc/lint/builtin.rs)
fn is_snake_case(ident: Ident) -> bool
{
    let ident = rust::get_ident(ident);
    assert!(!ident.get().is_empty());
    let ident = ident.get().trim_matches('_');

    let mut allow_underscore = true;
    ident.chars().all(|c| {
        allow_underscore = match c {
            c if c.is_lowercase() || c.is_numeric() => true,
            '_' if allow_underscore => false,
            _ => return false,
        };
        true
    })
}

// Adapted from the Rust compiler source (librustc/lint/builtin.rs)
fn to_snake_case(str: &str) -> String
{
    let mut words = vec![];
    for s in str.split('_') {
        let mut buf = String::new();
        if s.is_empty() { continue; }
        for ch in s.chars() {
            if !buf.is_empty() && ch.is_uppercase() {
                words.push(buf);
                buf = String::new();
            }
            buf.push(ch.to_lowercase());
        }
        words.push(buf);
    }
    words.connect("_")
}

pub fn id_to_snake_case(ident: Ident) -> String
{
  if !is_snake_case(ident.clone()) {
    to_snake_case(rust::get_ident(ident).get())
  } else {
    id_to_string(ident)
  }
}
