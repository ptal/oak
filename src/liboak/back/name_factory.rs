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

pub type Namespace = Vec<Ident>;

pub struct NameFactory<'cx>
{
  cx: &'cx ExtCtxt<'cx>,
  namespaces: Vec<Namespace>,
  mark_uid: usize
}

impl<'cx> NameFactory<'cx>
{
  pub fn new(cx: &'cx ExtCtxt) -> NameFactory<'cx> {
    NameFactory {
      cx: cx,
      namespaces: vec![],
      mark_uid: 0
    }
  }

  pub fn parser_name(&self, rule_name: Ident) -> Ident {
    self.ident_of(format!("parse_{}", id_to_string(rule_name)))
  }

  pub fn recognizer_name(&self, rule_name: Ident) -> Ident {
    self.ident_of(format!("recognize_{}", id_to_string(rule_name)))
  }

  pub fn next_mark_name(&mut self) -> Ident {
    self.mark_uid += 1;
    self.ident_of(format!("mark_{}", self.mark_uid))
  }

  pub fn next_data_name(&mut self) -> Ident {
    self.current_namespace().pop()
      .expect("Request a data name in an empty namespace.")
  }

  pub fn open_namespace(&mut self, cardinality: usize) -> Namespace {
    let namespace: Namespace = (0..cardinality)
      .map(|i| self.ident_of(format!("v_{}", i)))
      .collect();
    self.namespaces.push(namespace.clone());
    namespace
  }

  pub fn close_namespace(&mut self) {
    assert!(self.namespaces.len() > 0,
      "Try to close a namespace that has not been opened.");
    assert!(self.current_namespace().len() == 0,
      "Try to close a namespace that has not been fully consumed.");
    self.namespaces.pop();
  }

  fn current_namespace<'a>(&'a mut self) -> &'a mut Namespace {
    let len = self.namespaces.len();
    &mut self.namespaces[len - 1]
  }

  fn ident_of(&self, name: String) -> Ident {
    self.cx.ident_of(name.as_str())
  }
}
