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
use middle::attribute::invisible_type::*;
use middle::attribute::inline_type::*;

pub enum RuleTypeStyle
{
  New,
  Inline,
  Invisible
}

pub struct RuleType
{
  pub type_style: RuleTypeStyle
  // name: Option<RuleTypeNameAttribute>,
  // auto_gen_name: AutoGenName
}

// enum AutoGenName
// {
//   NoTransformation,
//   CamelCased
// }

// struct RuleTypeNameAttribute
// {
//   type_name: Option<String>,
//   fields_names: Vec<String>
// }

pub struct RuleTypeBuilder<'a>
{
  invisible_type_builder: InvisibleTypeBuilder<'a>,
  inline_type_builder: InlineTypeBuilder<'a>,
  cx: &'a ExtCtxt<'a>
}

impl<'a> RuleTypeBuilder<'a>
{
  pub fn new(cx: &'a ExtCtxt) -> RuleTypeBuilder<'a>
  {
    RuleTypeBuilder {
      invisible_type_builder: InvisibleTypeBuilder::new(cx),
      inline_type_builder: InlineTypeBuilder::new(cx),
      cx: cx
    }
  }

  pub fn from_attr(&mut self, attr: &rust::Attribute) -> bool
  {
    if self.invisible_type_builder.from_attr(attr) {
      return self.inline_type_builder.from_attr(attr)
    }
    false
  }

  pub fn build(&self) -> RuleType
  {
    let invisible_type = self.invisible_type_builder.build();
    let inline_type = self.inline_type_builder.build();
    if invisible_type && inline_type {
      self.cx.parse_sess.span_diagnostic.span_err(self.invisible_type_builder.invisible_type_attr.span,
        "Incoherent rule attributes: `invisible_type` and `inline_type` cannot be used \
        together. The attribute `invisible_type` makes the type of the rule invisible \
        so it is ignored by calling rules, instead, `inline_type` doesn't declare a new type for \
        the rule but its type is merged with the one of the calling rule.");
      self.cx.parse_sess.span_diagnostic.span_note(self.inline_type_builder.inline_type_attr.span,
        "`inline_type` attribute declared here.");
    }
    
    let type_style = 
      if invisible_type { Invisible }
      else if inline_type { Inline }
      else { New };

    RuleType {
      type_style: type_style
    }
  }
}
