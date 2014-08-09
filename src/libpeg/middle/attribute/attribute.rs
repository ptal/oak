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

pub use FGrammar = front::ast::Grammar;
pub use identifier::*;
pub use middle::attribute::code_printer::*;
pub use middle::attribute::code_gen::*;
pub use middle::attribute::rule_type::*;
pub use std::default::Default;
pub use rust;
pub use rust::ExtCtxt;

use attribute::model::AttributeDict;
use attribute::model_checker;

pub struct GrammarAttributes
{
  pub code_gen: CodeGeneration,
  pub code_printer: CodePrinter,
  pub starting_rule: Ident
  // lints: LintStore
}

impl GrammarAttributes
{
  fn register(model: &mut AttributeDict)
  {
    CodeGeneration::register(model);
    CodePrinter::register(model);
    // LintStore::register(model);
    // model.push(AttributeInfo::simple(
    //   "start",
    //   "entry point of the grammar, the parsing starts with this rule."
    // ));
  }

  pub fn new(cx: &ExtCtxt, first_rule: Ident, attributes: Vec<rust::Attribute>) -> GrammarAttributes
  {
    let mut model = AttributeDict::new(vec![]);
    GrammarAttributes::register(&mut model);
    let model = attributes.move_iter().fold(
      model, |model, attr| model_checker::check(cx, model, attr));

    GrammarAttributes {
      code_gen: CodeGeneration::new(&model),
      code_printer: CodePrinter::new(&model),
      starting_rule: first_rule
      // "First rule is by default considered as the starting point. \
      // Annotate the starting rule with `#[start]`."
    }
  }
}

pub struct RuleAttributes
{
  pub ty: RuleType
}

impl RuleAttributes
{
  fn register(model: &mut AttributeDict)
  {
    RuleType::register(model);
  }

  pub fn new(cx: &ExtCtxt, attributes: Vec<rust::Attribute>) -> RuleAttributes
  {
    let mut model = AttributeDict::new(vec![]);
    RuleAttributes::register(&mut model);
    let model = attributes.move_iter().fold(
      model, |model, attr| model_checker::check(cx, model, attr));

    RuleAttributes {
      ty: RuleType::new(cx, &model)
    }
  }
}
