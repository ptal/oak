// Copyright 2018 Chao Lin & William Sergeant (Sorbonne University)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![macro_use]
use std::char::*;
use middle::analysis::ast::*;
use self::Pattern::*;
use self::Character::*;

#[derive(Copy, Clone)]
enum Pattern {
  // OneOrMore, // +
  // ZeroOrMore, // *
  // ZeroOrOne, // ?
  One,
  // And, // &
  // Not, // !
}

#[derive(Copy, Clone)]
enum Character{
  Char(char),
  Any // .
}

struct Occurence
{
  choice: Vec<Vec<(Character,Pattern)>>
}

impl Occurence
{
  fn push_sequence(&self, other: Occurence) -> Occurence{
    let mut res = vec![];
    for choice1 in other.choice.clone(){
      for choice2 in self.choice.clone(){
        let mut tmp = choice2.clone();
        for couple in choice1.clone(){
          tmp.push(couple)
        }
        res.push(tmp)
      }
    }
    Occurence{
      choice: res
    }
  }

  fn merge_choice(&self, other: Occurence) -> Occurence {
    let mut res = self.choice.clone();
    for choice in other.choice.clone(){
      res.push(choice)
    }
    Occurence{
      choice: res
    }
  }

  fn copy(&self) -> Occurence{
    Occurence{
      choice: self.choice.to_vec()
    }
  }

  fn is_unreachable_with(&self, target: Occurence) -> bool {
    let mut res = true;
    for seq in self.choice.clone(){
      if !(target.success_with(seq)) {
        res = false;
        break;
      }
    }
    res
  }

  fn success_with(&self, seq2: Vec<(Character, Pattern)>) -> bool {
    let mut res = false;
    for seq1 in self.choice.clone(){
      if self.succeed_before(seq1,seq2.to_vec()) {
        res = true;
      }
    }
    res
  }

  fn succeed_before(&self, seq1: Vec<(Character, Pattern)>, seq2: Vec<(Character, Pattern)>) -> bool{
    let mut res = true;
    if seq2.len()<seq1.len() {
      res=false;
    }
    else{
      for (i,character_pattern1) in seq1.iter().enumerate(){
        if let Some(character_pattern2) = seq2.get(i) {
          let &(c1,_p1) = character_pattern1;
          let &(c2,_p2) = character_pattern2;
          match c1 {
            Char(character1) => {
              match c2 {
                Char(character2) => {
                  if character1!=character2 {
                    res=false;
                    break;
                  }
                }
                Any => {
                  res=false;
                  break;
                }
              }
            }
            Any => { continue; }
          }
        }
      }
    }
    res
  }
}

pub struct UnreachableRule<'a: 'c, 'b: 'a, 'c>
{
  grammar: &'c AGrammar<'a, 'b>
}

impl <'a, 'b, 'c> UnreachableRule<'a, 'b, 'c>
{
  pub fn analyse(grammar: AGrammar<'a, 'b>) -> Partial<AGrammar<'a, 'b>> {
    UnreachableRule::check_unreachable_rule(&grammar);
    Partial::Value(grammar)
  }

  fn check_unreachable_rule(grammar: &'c AGrammar<'a, 'b>){
    let mut analyser = UnreachableRule{
      grammar: grammar
    };

    for rule in &grammar.rules {
      analyser.visit_expr(rule.expr_idx);
    }
  }
}

impl<'a, 'b, 'c> ExprByIndex for UnreachableRule<'a, 'b, 'c>
{
  fn expr_by_index(&self, index: usize) -> Expression {
    self.grammar.expr_by_index(index).clone()
  }
}

impl<'a, 'b, 'c> Visitor<Occurence> for UnreachableRule<'a, 'b, 'c>
{
  fn visit_choice(&mut self, _this: usize, children: Vec<usize>) -> Occurence{
    let mut occurences_of_children = vec![];
    for child in children.clone(){
      let occ = self.visit_expr(child);
      if !occ.choice.is_empty() {
          occurences_of_children.push((occ,child));
      }
    }
    for (i, cp1) in occurences_of_children.iter().enumerate(){
      let (_left, right) = occurences_of_children.split_at(i+1);
      let &(ref occ1, child1) = cp1;
      for cp2 in right{
        let &(ref occ2, child2) = cp2;
        if occ2.is_unreachable_with(occ1.copy()) {
          self.grammar.span_warn(
            self.grammar[child2].span(),
            format!("This alternative will nerver succeed.")
          );
          self.grammar.span_note(
            self.grammar[child1].span(),
            format!("Because this alternative succeeded before reaching the previous one.")
          );
        }
      }
    }

    let mut res = vec![];
    if let Some((first, rest)) = children.split_first(){
      let mut occ = self.visit_expr(*first);
      for child in rest{
        occ = occ.merge_choice(self.visit_expr(*child))
      }
      res = occ.choice;
    }
    Occurence{
      choice: res
    }
  }

  fn visit_sequence(&mut self, _this: usize, children: Vec<usize>) -> Occurence{
    let mut res = vec![];
    if let Some((first, rest)) = children.split_first(){
      let mut occ = self.visit_expr(*first);
      for child in rest{
        occ = occ.push_sequence(self.visit_expr(*child))
      }
      res = occ.choice;
    }
    Occurence{
      choice: res
    }
  }

  fn visit_atom(&mut self, _this: usize) -> Occurence{
    Occurence{
      choice: vec![]
    }
  }

  fn visit_str_literal(&mut self, _this: usize, lit: String) -> Occurence{
    let mut seq = vec![];
    for c in lit.chars(){
      seq.push((Char(c),One))
    }
    let mut res = vec![];
    res.push(seq);
    Occurence{
      choice: res
    }
  }

  fn visit_any_single_char(&mut self, _this: usize) -> Occurence{
    let mut seq = vec![];
    seq.push((Any,One));
    let mut res = vec![];
    res.push(seq);
    Occurence{
      choice: res
    }
  }

  fn visit_character_class(&mut self, _this: usize, char_class: CharacterClassExpr) -> Occurence{
    let mut res = vec![];
    for intervals in char_class.intervals{
      for i in intervals.lo as u32 .. intervals.hi as u32 +1 {
        if let Some(c) = from_u32(i) {
          res.push(vec![(Char(c),One)])
        }
      }
    }
    Occurence{
      choice: res
    }
  }

  fn visit_non_terminal_symbol(&mut self, _this: usize, _rule: Ident) -> Occurence{
    Occurence{
      choice: vec![]
    }
  }

  fn visit_zero_or_more(&mut self, _this: usize, _child: usize) -> Occurence{
    Occurence{
      choice: vec![]
    }
  }

  fn visit_one_or_more(&mut self, _this: usize, _child: usize) -> Occurence{
    Occurence{
      choice: vec![]
    }
  }

  fn visit_optional(&mut self, _this: usize, _child: usize) -> Occurence{
    Occurence{
      choice: vec![]
    }
  }

  fn visit_not_predicate(&mut self, _this: usize, _child: usize) -> Occurence{
    Occurence{
      choice: vec![]
    }
  }

  fn visit_and_predicate(&mut self, _this: usize, _child: usize) -> Occurence{
    Occurence{
      choice: vec![]
    }
  }
}
