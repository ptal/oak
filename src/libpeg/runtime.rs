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

pub trait Parser
{
  fn parse<'a>(&self, input: &'a str) -> Result<Option<&'a str>, String>;
}

pub fn any_single_char(input: &str, pos: uint) -> Result<uint, String>
{
  if input.len() - pos > 0 {
    Ok(input.char_range_at(pos).next)
  } else {
    Err(format!("End of input when matching `.`"))
  }
}

pub fn match_literal(input: &str, pos: uint, lit: &str, lit_len: uint)
  -> Result<uint, String>
{
  if input.len() - pos == 0 {
    Err(format!("End of input when matching the literal `{}`", lit))
  } else if input.slice_from(pos).starts_with(lit) {
    Ok(pos + lit_len)
  } else {
    Err(format!("Expected `{}` but got `{}`", lit, input.slice_from(pos)))
  }
}

pub fn make_result<'a>(input: &'a str, parsing_res: &Result<uint, String>) 
 -> Result<Option<&'a str>, String>
{
  match parsing_res {
    &Ok(pos) => {
      assert!(pos <= input.len())
      if pos == input.len() {
        Ok(None) 
      } else {
        Ok(Some(input.slice_from(pos)))
      }
    },
    &Err(ref msg) => Err(msg.clone())
  }
}
