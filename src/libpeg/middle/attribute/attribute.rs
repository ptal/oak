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
pub use FRule = front::ast::Rule;
pub use identifier::*;
pub use middle::attribute::code_printer::*;
pub use middle::attribute::code_gen::*;
pub use middle::attribute::rule_type::*;
pub use std::default::Default;
pub use rust;
pub use rust::ExtCtxt;

use attribute::model::*;
use attribute::model_checker;
use attribute::compile_error::DuplicateAttribute;

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
  }

// StartingRule struct that takes Vec<Model> (of attributes).
// Extract model creation stuff outside of the new method. Just take models as arguments.
// Take grammar attributes and rules attributes models.

  pub fn new(cx: &ExtCtxt, rules: &Vec<FRule>, attributes: Vec<rust::Attribute>) -> GrammarAttributes
  {
    let mut model = AttributeDict::new(vec![]);
    GrammarAttributes::register(&mut model);
    let model = attributes.move_iter().fold(
      model, |model, attr| model_checker::check(cx, model, attr));
    let starting_rule = rules[0].name.node.clone();

    GrammarAttributes {
      code_gen: CodeGeneration::new(&model),
      code_printer: CodePrinter::new(&model),
      starting_rule: starting_rule
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
    model.push(AttributeInfo::simple(
      "start",
      "entry point of the grammar, the parsing starts with this rule."
    ))
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
