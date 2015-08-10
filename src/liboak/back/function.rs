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

use back::ast::*;
use back::ast::FunctionKind::*;
use back::naming::*;

pub struct FunctionGenerator<'cx>
{
  cx: &'cx ExtCtxt<'cx>,
  name_factory: NameFactory<'cx>,
  functions: HashMap<Ident, RItem>
}

impl<'cx> FunctionGenerator<'cx>
{
  pub fn new(cx: &'cx ExtCtxt) -> FunctionGenerator<'cx> {
    FunctionGenerator {
      cx: cx,
      name_factory: NameFactory::new(cx),
      functions: HashMap::new()
    }
  }

  fn generate_recognizer(&mut self, kind: FunctionKind, names: GenFunNames, recognizer_body: RExpr, public: bool) {
    if kind.is_recognizer() {
      self.push_unit_fun(names.recognizer, recognizer_body, public);
    }
  }

  fn generate_parser_alias(&mut self, kind: FunctionKind, names: GenFunNames, public: bool) -> bool {
    let GenFunNames{recognizer, parser} = names;
    if kind == ParserAlias {
      let recognizer_call = quote_expr!(self.cx, $recognizer(input, pos));
      self.push_unit_fun(parser, recognizer_call, public);
      true
    } else {
      false
    }
  }

  fn generate_parser(&mut self, kind: FunctionKind, names: GenFunNames, parser_body: RExpr, public: bool) {
    match kind {
      Parser(ty) | Both(ty) => {
        self.push_fun(names.parser, parser_body, ty, public);
      },
      _ => ()
    }
  }

  fn generate(&mut self, names: GenFunNames, kind: FunctionKind, recognizer_body: RExpr, parser_body: RExpr, public: bool) {
    self.generate_recognizer(kind.clone(), names, recognizer_body, public);
    if !self.generate_parser_alias(kind.clone(), names, public) {
      self.generate_parser(kind, names, parser_body, public);
    }
  }

  pub fn generate_expr(&mut self, expr_desc: &str, current_rule_id: Ident, kind: FunctionKind,
    recognizer_body: RExpr, parser_body: RExpr) -> GenFunNames
  {
    let names = self.name_factory.expression_name(expr_desc, current_rule_id);
    self.generate(names, kind, recognizer_body, parser_body, false);
    names
  }

  pub fn generate_unit_expr(&mut self, expr_desc: &str, current_rule_id: Ident, kind: FunctionKind,
    recognizer_body: RExpr) -> GenFunNames
  {
    assert!(kind.is_unit(),
      format!("Unit_expr: Expression `{}` is expected to have an unit type but found `{:?}`.", expr_desc, kind));
    let names = self.name_factory.expression_name(expr_desc, current_rule_id);
    self.generate_recognizer(kind.clone(), names, recognizer_body, false);
    self.generate_parser_alias(kind.clone(), names, false);
    names
  }

  pub fn generate_rule(&mut self, kind: FunctionKind, rule_id: Ident, expr_fn_names: GenFunNames) {
    let cx = self.cx;
    let rule_name = self.names_of_rule(rule_id);
    let GenFunNames{recognizer, parser} = expr_fn_names;
    self.generate(rule_name, kind,
      quote_expr!(cx, $recognizer(input, pos)),
      quote_expr!(cx, $parser(input, pos)),
      true
    )
  }

  pub fn names_of_rule(&mut self, rule_id: Ident) -> GenFunNames {
    self.name_factory.names_of_rule(rule_id)
  }

  fn push_fun(&mut self, name: Ident, body: RExpr, ty: RTy, public: bool) {

    let pub_kw = if public {
      Some(quote_tokens!(self.cx, pub))
    } else {
      None
    };
    let function = quote_item!(self.cx,
      $pub_kw fn $name(input: &str, pos: usize) -> oak_runtime::ParseResult<$ty>
      {
        $body
      }
    ).expect("Quotation of a generated function.");
    self.functions.insert(name, function);
  }

  fn push_unit_fun(&mut self, name: Ident, body: RExpr, public: bool) {
    self.push_fun(name, body, quote_ty!(self.cx, ()), public);
  }

  pub fn code(&mut self) -> Vec<RItem> {
    self.functions.drain().map(|(_,v)| v).collect()
  }
}
