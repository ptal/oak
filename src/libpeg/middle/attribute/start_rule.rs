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

pub struct StartRuleBuilder<'a>
{
  start_rule_attr: AttributeInfo<uint>,
  start: InternedString,
  cx: &'a ExtCtxt<'a>
}

impl<'a> StartRuleBuilder<'a>
{
  pub fn new(cx: &'a ExtCtxt) -> StartRuleBuilder<'a>
  {
    StartRuleBuilder {
      start_rule_attr: AttributeInfo::new(0),
      start: InternedString::new("start"),
      cx: cx
    }
  }

  pub fn from_attr(&mut self, rule_no: uint, attr: &rust::Attribute) -> bool
  {
    match attr.node.value.node {
      MetaWord(ref word) if *word == self.start => (),
      _ => return true
    };

    if self.start_rule_attr.has_value() {
      self.cx.parse_sess.span_diagnostic.span_err(attr.span,
        "Duplicate start attribute.");
      self.cx.parse_sess.span_diagnostic.span_note(self.start_rule_attr.span,
        "Previous declaration here.");
    } else {
      self.start_rule_attr.set(rule_no, attr.span);
    }
    false
  }

  pub fn build(&self) -> uint
  {
    if !self.start_rule_attr.has_value() {
      self.cx.parse_sess.span_diagnostic.handler.warn(
       "No rule has been specified as the starting point (attribute `#[start]`). \
        the first rule will be automatically considered as such.");
    }
    self.start_rule_attr.value_or_default()
  }
}
