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
use rust::Span;

pub struct AttributeInfo<A>
{
  pub value: Option<A>,
  pub span: Span,
  pub default: A
}

impl<A: Clone> AttributeInfo<A>
{
  pub fn new(default: A) -> AttributeInfo<A>
  {
    AttributeInfo {
      value: None,
      span: rust::DUMMY_SP,
      default: default
    }
  }

  pub fn has_value(&self) -> bool
  {
    self.value.is_some()
  }

  pub fn set(&mut self, value: A, span: Span)
  {
    self.value = Some(value);
    self.span = span;
  }

  pub fn value_or_default(&self) -> A
  {
    match self.value {
      None => self.default.clone(),
      Some(ref value) => value.clone()
    }
  }
}
