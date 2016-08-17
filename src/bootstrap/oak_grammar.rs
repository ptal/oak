// Copyright 2016 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// WARNING: This is not a current valid Oak grammar. This is design experimentation on the grammar's evolution.

pub use self~oak::*;

grammar! oak {
  type Stream = RustTokens;
  type Context = FGrammar;
  type Error = SyntaxError;

  extern context .alloc_expr -> usize;
  extern context .push_attributes -> ();
  extern context .push_rust_item -> ();
  extern context .push_rule -> ();
  extern action respan -> SpannedIdent;
  // extern parser ident -> Ident;

  grammar = block*

  block = rs_inner_attrs item

  rs_inner_attrs
    = ~inner_attributes > .push_attributes

  item
    = ~rs_item > .push_rust_item
    / rule

  rule
    = .. ~outer_attributes rule_decl "=" rule_rhs > .push_rule

  rule_decl
    = .. ~rs_ident > respan

  rule_rhs = .. choice > .alloc_expr

  choice
    = branch % "/" > Expression::Choice

  branch
    = sequence !">" !"->"
    / .. (sequence ">" ~rs_ident > SemanticAction) > .alloc_expr
    / .. (sequence "->" ty > TypeAscription) > .alloc_expr

  ty
    = "()" > IType::unit
    / "(^)" > IType::Invisible
    \ "" > UnknownTypeAscription

  sequence
    = syntactic_predicate !syntactic_predicate
    / .. (syntactic_predicate+ > Sequence) > .alloc_expr
    \ .. !syntactic_predicate > EmptyRuleBodyError

  syntactic_predicate
    = .. ("!" repeat > NotPredicate) > .alloc_expr
    / .. ("&" repeat > AndPredicate) > .alloc_expr
    \ .. ("!" / "&") !repeat > SyntacticPredicateExpectExpr
    / repeat

  repeat
    = .. (atom "*" > ZeroOrMore) > .alloc_expr
    / .. (atom "+" > OneOrMore) > .alloc_expr
    / .. (atom "?" > ZeroOrOne) > .alloc_expr
    / atom

  atom
    = .. (~cooked_literal > StrLiteral) > .alloc_expr
    / .. ("." > AnySingleChar) > .alloc_expr
    / "(" rule_rhs ")"
    / .. (!~parse_item !rule_lhs ~rs_ident > NonTerminalSymbol) > .alloc_expr
    / "[" char_class "]"

  char_class =
    \ .. !"]" > MissingStringLiteral
    / .. (~cooked_literal ~> ~char_class::char_range_set > CharacterClass) > .alloc_expr
}

grammar! char_class {
  type Context = FGrammar;
  type Error = SyntaxError;

  char_range_set
    = .. warning_both_sep? "-"? char_range* "-"? > CharacterClassExpr::full_range

  char_range =
    \ .. "-" &. > BadCharClassSeparator
    / . "-" . > CharacterInterval::new
    / !"-" . > CharacterInterval::single

  warning_both_sep -> (^)
    = &("-" char_range* "-") > WarningCharClassDoubleSep > .push_warning
}
