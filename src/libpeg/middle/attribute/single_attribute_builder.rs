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
use rust::{ExtCtxt, InternedString, MetaWord};
use middle::attribute::attribute::*;

pub struct SingleAttributeBuilder<'a, Value>
{
  pub attr_info: AttributeInfo<Value>,
  name: InternedString,
  cx: &'a ExtCtxt<'a>
}

impl<'a, Value: Clone> SingleAttributeBuilder<'a, Value>
{
  pub fn new(cx: &'a ExtCtxt, name: &'static str, default: Value) -> SingleAttributeBuilder<'a, Value>
  {
    SingleAttributeBuilder {
      attr_info: AttributeInfo::new(default),
      name: InternedString::new(name),
      cx: cx
    }
  }

  pub fn from_attr(&mut self, attr: &rust::Attribute, value: Value) -> bool
  {
    match attr.node.value.node {
      MetaWord(ref word) if *word == self.name => (),
      _ => return true
    };

    if self.attr_info.has_value() {
      self.cx.parse_sess.span_diagnostic.span_warn(attr.span,
        format!("Duplicate `{}` attribute.", self.name.get()).as_slice());
      self.cx.parse_sess.span_diagnostic.span_note(self.attr_info.span,
        "Previous declaration here.");
    } else {
      self.attr_info.set(value, attr.span);
    }
    false
  }

  pub fn build(&self) -> Value
  {
    self.attr_info.value_or_default()
  }
}
