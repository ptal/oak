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
use middle::typing::visitor::*;
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
  grammar.rules = ExpressionTyper::type_exprs(cx, &tgrammar.rules, rule_types);
  grammar
}

struct TypeGenerator;

impl TypeGenerator
{
  fn unit_ty(cx: &ExtCtxt) -> RTy {
    quote_ty!(cx, ())
  }

  fn vector_ty(cx: &ExtCtxt, ty: RTy) -> RTy {
    quote_ty!(cx, Vec<$ty>)
  }

  fn option_ty(cx: &ExtCtxt, ty: RTy) -> RTy {
    quote_ty!(cx, Option<$ty>)
  }

  fn char_ty(cx: &ExtCtxt) -> RTy {
    quote_ty!(cx, char)
  }

  fn action_ty(cx: &ExtCtxt, expr_ty: ExprTy) -> RTy {
    if let ExprTy::Action(return_ty) = expr_ty {
      match return_ty {
        NoReturn(_) | DefaultReturn(_) => TypeGenerator::unit_ty(cx),
        Return(ty) => ty
      }
    }
    else {
      panic!(format!("BUG: Expected `Action` type but found {:?}.", expr_ty));
    }
  }

  fn tuple_ty<F>(cx: &ExtCtxt, expr: &Box<TExpression>, mut rty_of_idx: F) -> RTy where
   F: FnMut(usize) -> RTy
  {
    // cx.ty(expr.span, rust::Ty_::TyTup(vec![]))
    let expr_ty = expr.ty_clone();
    if let ExprTy::Tuple(indexes) = expr_ty {
      let tys: Vec<_> = indexes.iter().map(|&idx| rty_of_idx(idx)).collect();
      if tys.len() == 1 {
        tys[0].clone()
      }
      else {
        cx.ty(expr.span, rust::Ty_::TyTup(tys))
      }
    }
    else {
      panic!("BUG: Expected `Tuple` type but found {:?}.", expr_ty);
    }
  }
}

struct ExpressionTyper<'a>
{
  cx: &'a ExtCtxt<'a>,
  rules_ty: HashMap<Ident, RTy>
}

impl<'a> ExpressionTyper<'a>
{
  fn type_exprs(cx: &'a ExtCtxt<'a>, rules: &'a HashMap<Ident, TRule>,
    rules_ty: HashMap<Ident, RTy>) -> HashMap<Ident, Rule>
  {
    let mut typer = ExpressionTyper {
      cx: cx,
      rules_ty: rules_ty
    };
    typer.visit_rules(rules)
  }

  fn visit_rules(&mut self, rules: &HashMap<Ident, TRule>) -> HashMap<Ident, Rule> {
    rules.iter()
    .map(|(&id, rule)| (id, self.visit_rule(rule)))
    .collect()
  }

  fn visit_rule(&mut self, rule: &TRule) -> Rule {
    Rule {
      name: rule.name,
      def: self.visit_expr(&rule.def)
    }
  }

  fn function_kind(&self, expr: &Box<TExpression>, ty: RTy) -> FunctionKind {
    match expr.context {
      EvaluationContext::UnValued => {
        FunctionKind::Recognizer
      }
      EvaluationContext::Both => {
        if expr.is_unit() {
          FunctionKind::ParserAlias
        }
        else {
          FunctionKind::Both(ty)
        }
      }
    }
  }

  fn build_expr(&self, parent: &Box<TExpression>, ty: RTy, node: ExpressionNode) -> Box<Expression> {
    box Expression {
      span: parent.span,
      ty: parent.ty_clone(),
      node: node,
      kind: self.function_kind(parent, ty)
    }
  }

  fn compose_expr<F,G>(&mut self, parent: &Box<TExpression>, expr: &Box<TExpression>,
    compose_ast: F, compose_ty: G) -> Box<Expression> where
   F: FnOnce(Box<Expression>) -> ExpressionNode,
   G: FnOnce(&ExtCtxt, RTy) -> RTy
  {
    let typed_expr = self.visit_expr(expr);
    let ty = typed_expr.return_type(self.cx);
    let parent_ty = compose_ty(self.cx, ty);
    self.build_expr(parent, parent_ty, compose_ast(typed_expr))
  }
}

impl<'a> Visitor<Box<Expression>> for ExpressionTyper<'a>
{
  fn visit_str_literal(&mut self, parent: &Box<TExpression>, lit: &String) -> Box<Expression> {
    let ty = TypeGenerator::unit_ty(self.cx);
    self.build_expr(parent, ty, StrLiteral(lit.clone()))
  }

  fn visit_not_predicate(&mut self, parent: &Box<TExpression>, expr: &Box<TExpression>) -> Box<Expression> {
    self.compose_expr(parent, expr, NotPredicate, |cx,_| TypeGenerator::unit_ty(cx))
  }

  fn visit_and_predicate(&mut self, parent: &Box<TExpression>, expr: &Box<TExpression>) -> Box<Expression> {
    self.compose_expr(parent, expr, AndPredicate, |cx,_| TypeGenerator::unit_ty(cx))
  }

  fn visit_any_single_char(&mut self, parent: &Box<TExpression>) -> Box<Expression> {
    let ty = TypeGenerator::char_ty(self.cx);
    self.build_expr(parent, ty, AnySingleChar)
  }

  fn visit_character_class(&mut self, parent: &Box<TExpression>, class: &CharacterClassExpr) -> Box<Expression> {
    let ty = TypeGenerator::char_ty(self.cx);
    self.build_expr(parent, ty, CharacterClass(class.clone()))
  }

  fn visit_character(&mut self, _parent: &Box<TExpression>) -> Box<Expression> {
    unreachable!();
  }

  fn visit_non_terminal_symbol(&mut self, parent: &Box<TExpression>, id: Ident) -> Box<Expression> {
    let ty = self.rules_ty[&id].clone();
    self.build_expr(parent, ty, NonTerminalSymbol(id))
  }

  fn visit_zero_or_more(&mut self, parent: &Box<TExpression>, expr: &Box<TExpression>) -> Box<Expression> {
    self.compose_expr(parent, expr, ZeroOrMore, TypeGenerator::vector_ty)
  }

  fn visit_one_or_more(&mut self, parent: &Box<TExpression>, expr: &Box<TExpression>) -> Box<Expression> {
    self.compose_expr(parent, expr, OneOrMore, TypeGenerator::vector_ty)
  }

  fn visit_optional(&mut self, parent: &Box<TExpression>, expr: &Box<TExpression>) -> Box<Expression> {
    self.compose_expr(parent, expr, Optional, TypeGenerator::option_ty)
  }

  fn visit_sequence(&mut self, parent: &Box<TExpression>, exprs: &Vec<Box<TExpression>>) -> Box<Expression> {
    let exprs = walk_exprs(self, exprs);
    let ty = TypeGenerator::tuple_ty(self.cx, parent, |idx| exprs[idx].return_type(self.cx));
    self.build_expr(parent, ty, Sequence(exprs))
  }

  fn visit_choice(&mut self, parent: &Box<TExpression>, exprs: &Vec<Box<TExpression>>) -> Box<Expression> {
    let exprs = walk_exprs(self, exprs);
    let ty = exprs[0].return_type(self.cx);
    self.build_expr(parent, ty, Choice(exprs))
  }

  fn visit_semantic_action(&mut self, parent: &Box<TExpression>, expr: &Box<TExpression>, id: Ident) -> Box<Expression> {
    self.compose_expr(parent, expr,
      |expr| SemanticAction(expr, id),
      |cx,_| TypeGenerator::action_ty(cx, parent.ty_clone()))
  }
}

struct RuleTyper<'a>
{
  cx: &'a ExtCtxt<'a>,
  rules: &'a HashMap<Ident, TRule>,
  visited: HashMap<Ident, bool>,
  rules_ty: HashMap<Ident, RTy>
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
      rules_ty: HashMap::with_capacity(rules.len())
    };
    typer.visit_rules(rules);
    typer.rules_ty
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
      self.rules_ty.insert(*ident, ty);
    }
  }
}

impl<'a> Visitor<RTy> for RuleTyper<'a>
{
  fn visit_expr(&mut self, expr: &Box<TExpression>) -> RTy {
    if expr.context == EvaluationContext::UnValued || expr.is_unit() {
      TypeGenerator::unit_ty(self.cx)
    }
    else {
      walk_expr(self, expr)
    }
  }

  fn visit_str_literal(&mut self, _parent: &Box<TExpression>, _lit: &String) -> RTy {
    panic!("BUG: String literal expression should have type `Unit` and handled in visit_expr.");
  }

  fn visit_syntactic_predicate(&mut self, _parent: &Box<TExpression>, _expr: &Box<TExpression>) -> RTy {
    panic!("BUG: Syntactic predicate (&e, !e) expressions should have type `Unit` and handled in visit_expr.");
  }

  fn visit_character(&mut self, _parent: &Box<TExpression>) -> RTy {
    TypeGenerator::char_ty(self.cx)
  }

  fn visit_non_terminal_symbol(&mut self, _parent: &Box<TExpression>, id: Ident) -> RTy {
    let rule = self.rules.get(&id).unwrap();
    self.visit_rule(rule);
    debug_assert!(self.rules_ty.contains_key(&id), "Try to use a type not yet computed. Probably a recursive type loop.");
    self.rules_ty[&id].clone()
  }

  fn visit_repeat(&mut self, _parent: &Box<TExpression>, expr: &Box<TExpression>) -> RTy {
    TypeGenerator::vector_ty(self.cx, walk_expr(self, expr))
  }

  fn visit_optional(&mut self, _parent: &Box<TExpression>, expr: &Box<TExpression>) -> RTy {
    TypeGenerator::option_ty(self.cx, walk_expr(self, expr))
  }

  fn visit_sequence(&mut self, parent: &Box<TExpression>, exprs: &Vec<Box<TExpression>>) -> RTy {
    TypeGenerator::tuple_ty(self.cx, parent, |idx| self.visit_expr(&exprs[idx]))
  }

  fn visit_choice(&mut self, _parent: &Box<TExpression>, exprs: &Vec<Box<TExpression>>) -> RTy {
    self.visit_expr(&exprs[0])
  }

  fn visit_semantic_action(&mut self, parent: &Box<TExpression>, _expr: &Box<TExpression>, _id: Ident) -> RTy {
    TypeGenerator::action_ty(self.cx, parent.ty_clone())
  }
}
