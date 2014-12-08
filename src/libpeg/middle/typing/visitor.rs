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

pub use middle::typing::ast::*;


use middle::typing::ast::ExpressionTypeVersion::*;
use middle::typing::ast::ExpressionType::*;
use middle::typing::ast::NamedExpressionType::*;

pub trait Visitor
{
  fn visit_rule(&mut self, rule: &Rule)
  {
    walk_rule(self, rule);
  }

  fn visit_rule_type(&mut self, ty: &PTy)
  {
    walk_ty(self, ty);
  }

  // fn visit_inlined_rule(&mut self, ty: &PTy)
  // {
  //   walk_ty(self, ty);
  // }

  // fn visit_new_rule(&mut self, ty: &Box<NamedExpressionType>)
  // {
  //   walk_named_ty(self, ty);
  // }

  fn visit_character(&mut self) {}
  fn visit_unit(&mut self) {}
  fn visit_unit_propagate(&mut self, _parent: &PTy) {}
  fn visit_rule_type_ph(&mut self, _parent: &PTy, _ident: Ident) {}

  // fn visit_named_type(&mut self, _name: &String, ty: &PTy)
  // {
  //   walk_ty(self, ty);
  // }

  fn visit_vector(&mut self, _parent: &PTy, inner: &PTy)
  {
    walk_ty(self, inner);
  }

  fn visit_tuple(&mut self, _parent: &PTy, inners: &Vec<PTy>)
  {
    walk_tys(self, inners);
  }

  fn visit_optional(&mut self, _parent: &PTy, inner: &PTy)
  {
    walk_ty(self, inner);
  }

  fn visit_unnamed_sum(&mut self, _parent: &PTy, inners: &Vec<PTy>)
  {
    walk_tys(self, inners);
  }

  // fn visit_struct(&mut self, _name: &String, fields: &Vec<(String, PTy)>)
  // {
  //   walk_named_tys(self, fields);
  // }

  // fn visit_struct_tuple(&mut self, _name: &String, fields: &Vec<PTy>)
  // {
  //   walk_tys(self, fields);
  // }

  // fn visit_sum(&mut self, _name: &String, variants: &Vec<(String, PTy)>)
  // {
  //   walk_named_tys(self, variants);
  // }

  // fn visit_type_alias(&mut self, _name: &String, ty: &PTy)
  // {
  //   walk_ty(self, ty);
  // }
}

pub fn walk_rule<V: Visitor>(visitor: &mut V, rule: &Rule)
{
  visitor.visit_rule_type(&rule.def.ty);
}

// pub fn walk_rule_type<V: Visitor>(visitor: &mut V, ty: &RuleType)
// {
//   match ty {
//     &InlineTy(ref ty) => visitor.visit_inlined_rule(ty),
//     &NewTy(ref ty) => visitor.visit_new_rule(ty)
//   }
// }

pub fn walk_ty<V: Visitor>(visitor: &mut V, ty: &PTy)
{
  // We don't want to borrow for the entire exploration, it'd
  // prevent mutable borrow.
  let ty_rc = {
    let ty_ref = ty.borrow();
    ty_ref.clone()
  };
  match &*ty_rc {
    &Character => visitor.visit_character(),
    &Unit => visitor.visit_unit(),
    &UnitPropagate => visitor.visit_unit_propagate(ty),
    &RuleTypePlaceholder(ref id) => visitor.visit_rule_type_ph(ty, id.clone()),
    &RuleTypeName(_) => (),
    &Vector(ref sub_ty) => visitor.visit_vector(ty, sub_ty),
    &Tuple(ref sub_tys) => visitor.visit_tuple(ty, sub_tys),
    &OptionalTy(ref sub_ty) => visitor.visit_optional(ty, sub_ty),
    &UnnamedSum(ref sub_tys) => visitor.visit_unnamed_sum(ty, sub_tys)
  }
}

// pub fn walk_named_ty<V: Visitor>(visitor: &mut V, ty: &Box<NamedExpressionType>)
// {
//   match ty {
//     &box Struct(ref name, ref named_tys) => visitor.visit_struct(name, named_tys),
//     &box StructTuple(ref name, ref tys) => visitor.visit_struct_tuple(name, tys),
//     &box Sum(ref name, ref named_tys) => visitor.visit_sum(name, named_tys),
//     &box TypeAlias(ref name, ref ty) => visitor.visit_type_alias(name, ty)
//   }
// }

pub fn walk_tys<V: Visitor>(visitor: &mut V, tys: &Vec<PTy>)
{
  for ty in tys.iter() {
    walk_ty(visitor, ty);
  }
}

// pub fn walk_named_tys<V: Visitor>(visitor: &mut V, tys: &Vec<(String, PTy)>)
// {
//   for &(ref name, ref ty) in tys.iter() {
//     visitor.visit_named_type(name, ty);
//   }
// }