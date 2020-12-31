// Copyright 2014 Pierre Talbot

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Up to this point, the parser automatically created `ExternalNonTerminalSymbol` for all rule's calls.
//! Here, we convert non terminal symbols that are declared in the current grammar to `NonTerminalSymbol`.

use middle::analysis::ast::*;
use partial::Partial::*;

pub struct ResolveNonTerminal
{
  grammar: AGrammar
}

impl ResolveNonTerminal
{
  pub fn resolve(grammar: AGrammar) -> Partial<AGrammar> {
    let rules_expr_idx: Vec<_> = grammar.rules.iter().map(|r| r.expr_idx).collect();
    let mut resolver = ResolveNonTerminal { grammar };
    for idx in rules_expr_idx {
      resolver.visit_expr(idx);
    }
    Value(resolver.grammar)
  }
}

impl ExprByIndex for ResolveNonTerminal
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index).clone()
  }
}

impl Visitor<()> for ResolveNonTerminal
{
  unit_visitor_impl!(sequence);
  unit_visitor_impl!(choice);

  fn visit_external_non_terminal_symbol(&mut self, this: usize, name: &syn::Path) {
    if let Some(ident) = name.get_ident() {
      let contains_key = self.grammar.rules.iter()
        .any(|r| r.ident() == ident.to_string());
      if contains_key {
        self.grammar.exprs[this] = Expression::NonTerminalSymbol(ident.clone());
      }
    }
  }
}
