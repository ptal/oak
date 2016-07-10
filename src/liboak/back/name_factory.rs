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

use identifier::*;
use rust::ExtCtxt;

pub fn parser_name(cx: &ExtCtxt, rule_name: Ident) -> Ident {
  string_to_ident(cx, format!("parse_{}", ident_to_string(rule_name)))
}

pub fn recognizer_name(cx: &ExtCtxt, rule_name: Ident) -> Ident {
  string_to_ident(cx, format!("recognize_{}", ident_to_string(rule_name)))
}

#[derive(Clone)]
pub struct Namespace
{
  vars: Vec<Ident>,
  num_vars_in_scope: usize
}

impl Namespace
{
  fn new(vars: Vec<Ident>) -> Namespace {
    let len = vars.len();
    Namespace {
      vars: vars,
      num_vars_in_scope: len
    }
  }

  pub fn empty() -> Namespace {
    Namespace::new(vec![])
  }

  pub fn fresh(cx: &ExtCtxt, cardinality: usize, var_prefix: String) -> Namespace {
    Namespace::new((0..cardinality)
      .map(|i| string_to_ident(cx, format!("{}{}", var_prefix, i)))
      .collect())
  }

  pub fn vars_in_scope(&self) -> Vec<Ident> {
    self.vars[0..self.num_vars_in_scope]
      .iter().cloned().collect()
  }

  pub fn next_unbounded_var(&mut self) -> Ident {
    assert!(self.has_next(), "Request a variable name in an empty namespace.");
    self.num_vars_in_scope -= 1;
    self.vars[self.num_vars_in_scope]
  }

  pub fn has_next(&self) -> bool {
    self.num_vars_in_scope > 0
  }
}

pub struct NameFactory
{
  namespaces: Vec<Namespace>,
  mark_uid: usize,
  label_uid: usize,
  closure_uid: usize
}

impl NameFactory
{
  pub fn new() -> NameFactory {
    NameFactory {
      namespaces: vec![Namespace::empty()],
      mark_uid: 0,
      label_uid: 0,
      closure_uid: 0
    }
  }

  pub fn next_mark_name(&mut self, cx: &ExtCtxt) -> Ident {
    self.mark_uid += 1;
    string_to_ident(cx, format!("mark_{}", self.mark_uid))
  }

  pub fn next_exit_label(&mut self, cx: &ExtCtxt) -> Ident {
    self.label_uid += 1;
    string_to_ident(cx, format!("'exit_{}", self.label_uid))
  }

  pub fn next_closure_name(&mut self, cx: &ExtCtxt) -> Ident {
    self.closure_uid += 1;
    string_to_ident(cx, format!("success_continuation_{}", self.closure_uid))
  }

  pub fn vars_in_scope(&self) -> Vec<Ident> {
    self.current_namespace().vars_in_scope()
  }

  pub fn next_unbounded_var(&mut self) -> Ident {
    self.current_namespace_mut().next_unbounded_var()
  }

  pub fn open_namespace(&mut self, cx: &ExtCtxt, cardinality: usize) -> Vec<Ident> {
    let prefix = self.current_var_prefix();
    self.namespaces.push(Namespace::fresh(cx, cardinality, prefix));
    self.current_namespace().vars_in_scope()
  }

  pub fn current_var_prefix(&self) -> String {
    const LETTERS: &'static [u8] = b"abcdefghijklmnopqrstuvwxyz";
    let mut prefix = String::new();
    let mut num_letter = self.namespaces.len();
    while num_letter > 0 {
      let x = (num_letter - 1) % LETTERS.len();
      num_letter /= LETTERS.len();
      prefix.push(LETTERS[x] as char);
    }
    prefix
  }

  pub fn close_namespace(&mut self) {
    assert!(self.namespaces.len() > 1,
      "close_namespace: There is no namespace opened.");
    assert!(!self.current_namespace().has_next(),
      "Try to close a namespace that has not been fully consumed.");
    self.namespaces.pop();
  }

  fn current_namespace<'a>(&'a self) -> &'a Namespace {
    let last = self.namespaces.len() - 1;
    &self.namespaces[last]
  }

  fn current_namespace_mut<'a>(&'a mut self) -> &'a mut Namespace {
    let last = self.namespaces.len() - 1;
    &mut self.namespaces[last]
  }

  pub fn save_namespace(&self) -> usize {
    self.current_namespace().num_vars_in_scope
  }

  pub fn restore_namespace(&mut self, num_vars_in_scope: usize) {
    assert!(self.current_namespace().vars.len() >= num_vars_in_scope,
      "Mismatch between the number of variable in scope and the current namespace.");
    self.current_namespace_mut().num_vars_in_scope = num_vars_in_scope;
  }
}
