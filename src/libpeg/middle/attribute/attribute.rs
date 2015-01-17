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

pub use identifier::*;
pub use middle::attribute::code_printer::*;
pub use middle::attribute::code_gen::*;
pub use middle::attribute::rule_type::*;

use rust::ExtCtxt;
use attribute::model::{AttributeArray, access, DuplicateAttribute, AttributeMerger, AttributeInfo};

pub struct GrammarAttributes
{
  pub code_gen: CodeGeneration,
  pub code_printer: CodePrinter,
  pub starting_rule: Ident
  // lints: LintStore
}

impl GrammarAttributes
{
  pub fn model() -> AttributeArray
  {
    let mut model = CodeGeneration::model();
    model.extend(CodePrinter::model().into_iter());
    model
  }

  pub fn new(cx: &ExtCtxt, rules_attrs: Vec<(Ident, AttributeArray)>,
    grammar_attrs: AttributeArray) -> GrammarAttributes
  {
    GrammarAttributes {
      code_gen: CodeGeneration::new(&grammar_attrs),
      code_printer: CodePrinter::new(&grammar_attrs),
      starting_rule: GrammarAttributes::starting_rule(cx, rules_attrs)
    }
  }

  fn starting_rule(cx: &ExtCtxt, rules_attrs: Vec<(Ident, AttributeArray)>) -> Ident
  {
    let mut start_name = None;
    for &(ref name, ref attr) in rules_attrs.iter() {
      if access::plain_value(attr, "start").has_value() {
        start_name = Some(name.clone());
      }
    }
    match start_name {
      None => GrammarAttributes::starting_rule_default(cx, rules_attrs),
      Some(name) => {
        GrammarAttributes::check_start_duplicate(cx, rules_attrs);
        name
      }
    }
  }

  fn check_start_duplicate(cx: &ExtCtxt, rules_attrs: Vec<(Ident, AttributeArray)>)
  {
    let duplicate = DuplicateAttribute::error(
      "There is only one starting rule per grammar.");
    let merger = AttributeMerger::new(cx, duplicate);
    let mut rules_iter = rules_attrs.into_iter().map(|(_, attr)| attr);
    let first = rules_iter.next().unwrap();
    rules_iter.fold(start_by_name(first),
      |accu, attr| merger.merge(accu, start_by_name(attr)));
  }

  fn starting_rule_default(cx: &ExtCtxt, rules_attrs: Vec<(Ident, AttributeArray)>) -> Ident
  {
    cx.parse_sess.span_diagnostic.handler.warn(
      "No rule has been specified as the starting point (attribute `#[start]`). \
       The first rule will be automatically considered as such.");
    let (name, _) = rules_attrs[0];
    name
  }
}

fn start_by_name(attr: AttributeArray) -> AttributeInfo
{
  access::by_name(&attr, "start").clone()
}

pub struct RuleAttributes
{
  pub ty: RuleType
}

impl RuleAttributes
{
  pub fn model() -> AttributeArray
  {
    let mut model = RuleType::model();
    model.push(AttributeInfo::simple(
      "start",
      "entry point of the grammar, the parsing starts with this rule."
    ));
    model
  }

  pub fn new(cx: &ExtCtxt, rule_attr: &AttributeArray) -> RuleAttributes
  {
    RuleAttributes {
      ty: RuleType::new(cx, rule_attr)
    }
  }
}
