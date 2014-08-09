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

pub use rust::{ExtCtxt, Span};
use attribute::model::*;

pub enum RuleTypeStyle
{
  New,
  Inline(Span),
  Invisible(Span)
}

impl RuleTypeStyle
{
  pub fn new(cx: &ExtCtxt, model: &AttributeDict) -> RuleTypeStyle
  {
    let inline_type = model.plain_value("inline_type");
    let invisible_type = model.plain_value("invisible_type");
    let inline = inline_type.value_or(false);
    let invisible = invisible_type.value_or(false);
    if inline && invisible {
      cx.parse_sess.span_diagnostic.span_err(inline_type.span(),
        "Incoherent rule type specifiers, a rule can't be inlined and invisible.");
      cx.parse_sess.span_diagnostic.span_note(invisible_type.span(),
        "Second incoherent rule type specifiers declared here.");
      New
    } else if inline {
      Inline(inline_type.span())
    } else if invisible {
      Invisible(invisible_type.span())
    } else {
      New
    }
  }

  pub fn register(model: &mut AttributeDict)
  {
    model.push_all(vec![
      AttributeInfo::simple(
        "inline_type",
        "the type of the rule will be merged with the type of the calling site. No rule type will be created.",
      ),
      AttributeInfo::simple(
        "invisible_type",
        "the calling site will ignore the type of this rule. The AST of the calling rule will not reference this rule.",
      )
    ]);
  }
}

pub struct RuleType
{
  pub type_style: RuleTypeStyle
  // name: Option<RuleTypeNameAttribute>,
  // auto_gen_name: AutoGenName
}

impl RuleType
{
  pub fn new(cx: &ExtCtxt, model: &AttributeDict) -> RuleType
  {
    RuleType {
      type_style: RuleTypeStyle::new(cx, model)
    }
  }

  pub fn register(model: &mut AttributeDict)
  {
    RuleTypeStyle::register(model);
  }
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

// fn make_invisible_type_builder<'a>(cx: &'a ExtCtxt) -> SingleAttributeBuilder<'a, bool>
// {
//   SingleAttributeBuilder::new(cx, "invisible_type", false)
// }

// fn make_inline_type_builder<'a>(cx: &'a ExtCtxt) -> SingleAttributeBuilder<'a, bool>
// {
//   SingleAttributeBuilder::new(cx, "inline_type", false)
// }

// pub struct RuleTypeBuilder<'a>
// {
//   invisible_type_builder: SingleAttributeBuilder<'a, bool>,
//   inline_type_builder: SingleAttributeBuilder<'a, bool>,
//   cx: &'a ExtCtxt<'a>
// }

// impl<'a> RuleTypeBuilder<'a>
// {
//   pub fn new(cx: &'a ExtCtxt) -> RuleTypeBuilder<'a>
//   {
//     RuleTypeBuilder {
//       invisible_type_builder: make_invisible_type_builder(cx),
//       inline_type_builder: make_inline_type_builder(cx),
//       cx: cx
//     }
//   }

//   pub fn from_attr(&mut self, attr: &rust::Attribute) -> bool
//   {
//     if self.invisible_type_builder.from_attr(attr, true) {
//       return self.inline_type_builder.from_attr(attr, true)
//     }
//     false
//   }

//   pub fn build(&self) -> RuleType
//   {
//     let invisible_type = self.invisible_type_builder.build();
//     let inline_type = self.inline_type_builder.build();
//     if invisible_type && inline_type {
//       self.cx.parse_sess.span_diagnostic.span_err(self.invisible_type_builder.attr_info.span,
//         "Incoherent rule attributes: `invisible_type` and `inline_type` cannot be used \
//         together. The attribute `invisible_type` makes the type of the rule invisible \
//         so it is ignored by calling rules, instead, `inline_type` doesn't declare a new type for \
//         the rule but its type is merged with the one of the calling rule.");
//       self.cx.parse_sess.span_diagnostic.span_note(self.inline_type_builder.attr_info.span,
//         "`inline_type` attribute declared here.");
//     }

//     let type_style = 
//       if invisible_type { Invisible }
//       else if inline_type { Inline }
//       else { New };

//     RuleType {
//       type_style: type_style
//     }
//   }
// }
