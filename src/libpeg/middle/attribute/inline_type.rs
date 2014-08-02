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

pub struct InlineTypeBuilder<'a>
{
  pub inline_type_attr: AttributeInfo<bool>,
  inline_type: InternedString,
  cx: &'a ExtCtxt<'a>
}

impl<'a> InlineTypeBuilder<'a>
{
  pub fn new(cx: &'a ExtCtxt) -> InlineTypeBuilder<'a>
  {
    InlineTypeBuilder {
      inline_type_attr: AttributeInfo::new(false),
      inline_type: InternedString::new("inline_type"),
      cx: cx
    }
  }

  pub fn from_attr(&mut self, attr: &rust::Attribute) -> bool
  {
    match attr.node.value.node {
      MetaWord(ref word) if *word == self.inline_type => (),
      _ => return true
    };

    if self.inline_type_attr.has_value() {
      self.cx.parse_sess.span_diagnostic.span_warn(attr.span,
        "Duplicate inline_type attribute.");
      self.cx.parse_sess.span_diagnostic.span_note(self.inline_type_attr.span,
        "Previous declaration here.");
    } else {
      self.inline_type_attr.set(true, attr.span);
    }
    false
  }

  pub fn build(&self) -> bool
  {
    self.inline_type_attr.value_or_default()
  }
}
