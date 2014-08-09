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

impl Default for RuleAttributes
{
  fn default() -> RuleAttributes
  {
    RuleAttributes {
      ty: Default::default()
    }
  }
}

// impl Attributes
// {
//   pub fn register(attr_dict: &mut AttributeDict)
//   {
//     GrammarAttributes::register(attr_dict);
//     RuleAttributes::register(attr_dict);
//   }

//   pub fn build(cx: &'a ExtCtxt<'a>, grammar: &'a Grammar) -> Attributes
//   {
//     let mut attribute_dict = AttributeDict::new();

//     let mut grammar_builders = vec![
//       box CodeGenerationBuilder::new(cx),
//       box CodePrinterBuilder::new(cx)
//     ]

//     let mut grammar_from_rule_builders = vec![
//       box StartRuleBuilder::new(cx)
//     ]

//     for builder in rule_builders.iter().chain(
//                    grammar_builders.iter())
//     {
//       builder.register_attr(attribute_dict);
//     }

//     grammar.rules.iter().map(|r| {
//       let mut rule_builders = vec![
//         box RuleTypeBuilder::new(cx)
//       ];
//       let _rules_attrs : Vec<&Attribute> = r.attributes.iter()
//         .filter(|&a| filter_all(rule_builders, a))
//         .collect();
//       rule_builders.mut_iter().fold(RuleAttributes::default(),
//         |attr, builder| builder.build(attr))
//       // Check here for unused attributes.
//     }
//   }

//   fn filter_all(builders: &mut Vec<Box<Builder>>, attr: &Attribute) -> bool
//   {
//     for builder in builders {
//       if !builder.from_attr(attr) {
//         return false
//       }
//     }
//     true
//   }
// }
