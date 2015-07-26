// Copyright 2015 Pierre Talbot (IRCAM)

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
use rust::ast::FunctionRetTy::*;
use rust::AstBuilder;
use middle::ast::Grammar as TGrammar;
use middle::ast::Rule as TRule;
use middle::ast::Expression as TExpression;
use middle::ast::EvaluationContext;
use back::ast::*;
use back::ast::Expression_::*;

pub fn generate_rust_types(cx: &ExtCtxt, tgrammar: TGrammar) -> Grammar
{
  let mut grammar = Grammar {
    name: tgrammar.name,
    rules: HashMap::with_capacity(tgrammar.rules.len()),
    rust_items: tgrammar.rust_items,
    attributes: tgrammar.attributes
  };
  let rule_types = RuleTyper::type_rules(cx, &tgrammar.rules);
  grammar
}

struct RuleTyper<'a>
{
  cx: &'a ExtCtxt<'a>,
  rules: &'a HashMap<Ident, TRule>,
  visited: HashMap<Ident, bool>,
  types: HashMap<Ident, RTy>
}

impl<'a> RuleTyper<'a>
{
  fn type_rules(cx: &'a ExtCtxt<'a>, rules: &'a HashMap<Ident, TRule>) -> HashMap<Ident, RTy>
  {
    let mut visited = HashMap::with_capacity(rules.len());
    for id in rules.keys() {
      visited.insert(*id, false);
    }
    let mut typer = RuleTyper {
      cx: cx,
      rules: rules,
      visited: visited,
      types: HashMap::with_capacity(rules.len())
    };
    typer.visit_rules(rules);
    typer.types
  }

  fn visit_rules(&mut self, rules: &HashMap<Ident, TRule>) {
    for rule in rules.values() {
      self.visit_rule(rule);
    }
  }

  fn visit_rule(&mut self, rule: &TRule) {
    let ident = &rule.name.node;
    if !*self.visited.get(ident).unwrap() {
      *self.visited.get_mut(ident).unwrap() = true;
      let ty = self.visit_expr(&rule.def);
      self.types.insert(*ident, ty);
    }
  }

  fn visit_expr(&mut self, expr: &Box<TExpression>) -> RTy {
    let unit_ty = quote_ty!(self.cx, ());
    if expr.context == EvaluationContext::UnValued || expr.is_unit() {
      unit_ty
    }
    else {
      let parent_ty = expr.ty_clone();
      match &expr.node {
          &ZeroOrMore(ref sub)
        | &OneOrMore(ref sub) => {
          let sub_ty = self.visit_expr(sub);
          quote_ty!(self.cx, Vec<$sub_ty>)
        },
        &Optional(ref sub) => {
          let sub_ty = self.visit_expr(sub);
          quote_ty!(self.cx, Option<$sub_ty>)
        },
        &Choice(ref subs) => {
          self.visit_expr(&subs[0])
        },
        &SemanticAction(_, _) => {
          if let ExprTy::Action(rust_ty) = parent_ty {
            match rust_ty {
              NoReturn(_) | DefaultReturn(_) => unit_ty,
              Return(ty) => ty
            }
          }
          else {
            panic!("BUG: Semantics action does not have type `Unit` or `Action`.");
          }
        },
        &NonTerminalSymbol(id) => self.visit_non_terminal(expr, id),
        &Sequence(ref subs) => {
          if let ExprTy::Tuple(indexes) = parent_ty {
            let tys: Vec<_> = indexes.iter().map(|&idx| self.visit_expr(&subs[idx])).collect();
            if tys.len() == 1 {
              tys[0].clone()
            }
            else {
              self.cx.ty(expr.span, rust::Ty_::TyTup(tys))
            }
          }
          else {
            panic!("BUG: Sequence does not have type `Unit` or `Tuple`.");
          }
        }
        _ => {
          panic!("BUG: Expression should have type `Unit`");
        }
      }
    }
  }

  fn visit_non_terminal(&mut self, parent: &Box<TExpression>, id: Ident) -> RTy
  {
    let rule = self.rules.get(&id).unwrap();
    self.visit_rule(rule);
    self.types[&id].clone()
  }
}
