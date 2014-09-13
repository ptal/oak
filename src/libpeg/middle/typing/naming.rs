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

use middle::typing::visitor::*;
use middle::typing::ast::*;

pub fn naming_phase(cx: &ExtCtxt, grammar: &mut Grammar)
{
}


// fn infer_inline_rule_type(name: SpannedIdent, expr: Box<Expression>) -> Rule
// {
//   let ty = expr.ty.clone();
//   Rule{
//     name: name,
//     def: expr,
//     ty: InlineTy(ty)
//   }
// }

// fn infer_invisible_rule_type(name: SpannedIdent, expr: Box<Expression>) -> Rule
// {
//   Rule{
//     name: name,
//     def: expr,
//     ty: InlineTy(Rc::new(UnitPropagate))
//   }
// }

// fn infer_new_rule_type(cx: &ExtCtxt, name: SpannedIdent, expr: Box<Expression>) -> Rule
// {
//   let rule_ty = infer_rule_type_structure(cx, name.node.clone(), &expr);
//   Rule{
//     name: name,
//     def: expr,
//     ty: rule_ty
//   }
// }

// fn infer_rule_type_structure(cx: &ExtCtxt, rule_name: Ident, expr: &Box<Expression>) -> RuleType
// {
//   match &expr.node {
//     &Choice(ref expr) => named_choice_of_rule(cx, rule_name, expr),
//     &Sequence(_) => named_sequence_of_rule(rule_name, &expr.ty),
//     _ => type_alias_of_rule(rule_name, expr.ty.clone())
//   }
// }

// fn named_choice_of_rule(cx: &ExtCtxt, rule_name: Ident, exprs: &Vec<Box<Expression>>) -> RuleType
// {
//   let mut branches = vec![];
//   for expr in exprs.iter() {
//     let ty = expr.ty.clone();
//     match &*expr.ty {
//       &RuleTypePlaceholder(ref ident) |
//       &RuleTypeName(ref ident) => 
//         branches.push((name_of_sum(ident.clone()), ty)),
//       _ => {
//         cx.span_err(expr.span.clone(), format!("Name missing from this expression. Name is \
//           needed to build the AST of the current choice statement.").as_slice());
//       }
//     }
//   }
//   NewTy(box Sum(name_of_sum(rule_name), branches))
// }

// fn name_of_sum(ident: Ident) -> String
// {
//   id_to_camel_case(ident)
// }

// fn named_sequence_of_rule(rule_name: Ident, ty: &PTy) -> RuleType
// {
//   match &**ty {
//     &Tuple(ref tys) => NewTy(named_seq_tuple_of_rule(rule_name, tys)),
//     &Unit => InlineTy(Rc::new(Unit)),
//     &UnitPropagate => InlineTy(Rc::new(UnitPropagate)),
//     _ => type_alias_of_rule(rule_name, ty.clone())
//   }
// }

// fn named_seq_tuple_of_rule(rule_name: Ident,
//   tys: &Vec<PTy>) -> Box<NamedExpressionType>
// {
//   if tys.iter().all(|ty| ty.is_type_ph()) {
//     let names_tys = tys.iter()
//       .map(|ty| (id_to_snake_case(ty.ph_ident()), ty.clone()))
//       .collect();
//     box Struct(type_name_of_rule(rule_name), names_tys)
//   } else {
//     box StructTuple(type_name_of_rule(rule_name), tys.clone())
//   }
// }

// fn type_alias_of_rule(rule_name: Ident, ty: PTy) -> RuleType
// {
//   NewTy(box TypeAlias(type_name_of_rule(rule_name), ty))
// }

// fn type_name_of_rule(rule_name: Ident) -> String
// {
//   id_to_camel_case(rule_name)
// }


// fn type_of_choice_expr(&self, exprs: &Vec<Box<Expression>>) -> Option<Box<ExpressionType>>
// {
//   fn flatten_tuple(ty: Box<ExpressionType>) -> Vec<Box<ExpressionType>>
//   {
//     match ty {
//       box Tuple(tys) => tys,
//       _ => vec![ty]
//     }
//   };

//   let ty = exprs.iter()
//     .map(|expr| self.type_of_expr(expr))
//     .map(|ty| ty.map_or(vec![], flatten_tuple))
//     .map(|tys| box SumBranch(tys))
//     .collect();

//   Some(box Sum(ty))
// }
