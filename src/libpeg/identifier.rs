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

use rust;
use std::ops::Deref;
pub use std::string::String;
pub use rust::{Ident, Name};

pub fn id_to_string(id: Ident) -> String
{
  String::from(rust::get_ident(id).deref())
}

pub fn name_to_string(name: Name) -> String
{
  String::from(rust::get_name(name).deref())
}

pub fn string_to_lowercase(s: &String) -> String
{
  s.chars().flat_map(char::to_lowercase).collect()
}

pub fn ident_to_lowercase(ident: Ident) -> String
{
  let ident = id_to_string(ident);
  string_to_lowercase(&ident)
}
