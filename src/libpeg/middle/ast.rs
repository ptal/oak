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

pub use front::ast::{Expression_, Expression, CharacterInterval, CharacterClassExpr};
pub use front::ast::{
  StrLiteral, AnySingleChar, NonTerminalSymbol, Sequence,
  Choice, ZeroOrMore, OneOrMore, Optional, NotPredicate,
  AndPredicate, CharacterClass};

pub use rust::{ExtCtxt, Span, Spanned, SpannedIdent};
pub use middle::attribute::attribute::*;
pub use identifier::*;
pub use std::collections::hashmap::HashMap;

pub use FGrammar = front::ast::Grammar;
use FRule = front::ast::Rule;

pub struct Grammar{
  pub name: Ident,
  pub rules: HashMap<Ident, Rule>,
  pub attributes: GrammarAttributes
}

impl Grammar
{
  pub fn new(cx: &ExtCtxt, fgrammar: FGrammar) -> Option<Grammar>
  {
    let attributes = GrammarAttributes::new(cx,
      fgrammar.rules[0].name.node.clone(), fgrammar.attributes);
    let name = fgrammar.name;
    Grammar::make_rules(cx, fgrammar.rules).map(|rules|
      Grammar {
        name: name,
        rules: rules,
        attributes: attributes
      }
    )
  }

  fn make_rules(cx: &ExtCtxt, rules: Vec<FRule>) -> Option<HashMap<Ident, Rule>>
  {
    let mut rules_map = HashMap::with_capacity(rules.len());
    let rules_len = rules.len();
    for rule in rules.move_iter() {
      let rule_name = rule.name.node.clone();
      if !rules_map.contains_key(&rule_name) {
        Rule::new(cx, rule).map(|rule|
            rules_map.insert(rule_name, rule));
      } else {
        Grammar::duplicate_rules(cx, 
          rules_map.get(&rule_name).name.span, rule.name.span)
      }
    }
    // If the lengths differ, an error occurred.
    Some(rules_map).filtered(|id2rule|
      id2rule.len() == rules_len)
  }

  fn duplicate_rules(cx: &ExtCtxt, pre: Span, current: Span)
  {
    cx.span_err(current, "Duplicate rule definition.");
    cx.span_note(pre, "Previous declaration here.");
  }
}

pub struct Rule{
  pub name: SpannedIdent,
  pub attributes: RuleAttributes,
  pub def: Box<Expression>,
}

impl Rule
{
  fn new(cx: &ExtCtxt, frule: FRule) -> Option<Rule>
  {
    let attributes = RuleAttributes::new(cx, frule.attributes);
    Some(Rule{
      name: frule.name,
      attributes: attributes,
      def: frule.def
    })
  }
}