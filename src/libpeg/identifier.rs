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
