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
use rust::ExtCtxt;
use middle::attribute::single_attribute_builder::SingleAttributeBuilder;

pub struct StartRuleBuilder<'a>
{
  builder: SingleAttributeBuilder<'a, uint>,
  cx: &'a ExtCtxt<'a>
}

impl<'a> StartRuleBuilder<'a>
{
  pub fn new(cx: &'a ExtCtxt) -> StartRuleBuilder<'a>
  {
    StartRuleBuilder {
      builder: SingleAttributeBuilder::new(cx, "start", 0),
      cx: cx
    }
  }

  pub fn from_attr(&mut self, attr: &rust::Attribute, rule_no: uint) -> bool
  {
    self.builder.from_attr(attr, rule_no)
  }

  pub fn build(&self) -> uint
  {
    if !self.builder.attr_info.has_value() {
      self.cx.parse_sess.span_diagnostic.handler.warn(
       "No rule has been specified as the starting point (attribute `#[start]`). \
        the first rule will be automatically considered as such.");
    }
    self.builder.build()
  }
}
