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
pub use middle::attribute::rule_type::RuleTypeStyle::*;
use attribute::model::*;
use attribute::compile_error::CompileErrorLevel::*;
use attribute::model::AttributeLitModel::*;
use attribute::model::AttributeModel::*;

#[derive(Clone)]
pub enum RuleTypeStyle
{
  New,
  Inline(Span),
  Invisible(Span)
}

impl RuleTypeStyle
{
  pub fn new(cx: &ExtCtxt, model: &AttributeArray) -> RuleTypeStyle
  {
    let inline_type = access::plain_value(model, "inline_type");
    let invisible_type = access::plain_value(model, "invisible_type");
    let inline = inline_type.has_value() || false;
    let invisible = invisible_type.has_value() || false;
    if inline && invisible {
      cx.span_err(inline_type.span(),
        "Incoherent rule type specifiers, a rule can't be inlined and invisible.");
      cx.span_note(invisible_type.span(),
        "Previous declaration here.");
      New
    } else if inline {
      Inline(inline_type.span())
    } else if invisible {
      Invisible(invisible_type.span())
    } else {
      New
    }
  }

  pub fn model() -> AttributeArray
  {
    vec![
      AttributeInfo::simple(
        "inline_type",
        "the type of the rule will be merged with the type of the calling site. No rule type will be created.",
      ),
      AttributeInfo::simple(
        "invisible_type",
        "the calling site will ignore the type of this rule. The AST of the calling rule will not reference this rule.",
      )
    ]
  }
}

pub struct RuleType
{
  pub style: RuleTypeStyle,
  pub _name: Option<ComposedTypeName>
}

impl RuleType
{
  pub fn new(cx: &ExtCtxt, model: &AttributeArray) -> RuleType
  {
    RuleType {
      style: RuleTypeStyle::new(cx, model),
      _name: ComposedTypeName::new(cx, model)
    }
  }

  pub fn model() -> AttributeArray
  {
    let mut model = RuleTypeStyle::model();
    model.extend(ComposedTypeName::model().into_iter());
    model
  }
}

// If the name must be infered, than name = None.
struct ComposedTypeName
{
  _name: Option<String>,
  _fields_names: Vec<ComposedTypeName>
}

impl ComposedTypeName
{
  pub fn new(_cx: &ExtCtxt, _model: &AttributeArray) -> Option<ComposedTypeName>
  {
    None
    // let value = access::lit_str(model, "type_name");
    // let span = value.span;
    // value.value.map(|(val, _)| ComposedTypeName::from_str(cx, span, val.get()))
  }

  // fn from_str(cx: &ExtCtxt, span: Span, val: &str) -> ComposedTypeName
  // {

  // }

  pub fn model() -> AttributeArray
  {
    vec![
      AttributeInfo::new(
        "type_name",
        "the name of the generated type. `#[type_name = \"EnumName(EnumField, EnumField2)\"]` is \
        for a rule with the shape `rule = r1 / r2`. It also works for structure such as \
        `StructName(field1, field2)` if the rule have the shape `rule = r1 r2`. Use `_` if \
        you want the name to be inferred.",
        KeyValue(MLitStr(AttributeValue::new(DuplicateAttribute::simple(Error))))
      )
    ]
  }
}
