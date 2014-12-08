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

pub use std::default::Default;
use attribute::model::*;
use attribute::model::AttributeModel::*;

pub struct CodePrinter
{
  pub info: bool,
  pub ast: bool,
  pub parser: bool
}

impl CodePrinter
{
  pub fn new(model: &AttributeArray) -> CodePrinter
  {
    let model = access::sub_model(model, "print");
    let ast = access::plain_value_or(model, "ast", false);
    let parser = access::plain_value_or(model, "parser", false);
    let info = access::plain_value_or(model, "info", false);
    let code = access::plain_value_or(model, "code", false);
    let all = access::plain_value_or(model, "all", false);
    CodePrinter {
      ast: ast || code || all,
      parser: parser || code || all,
      info: info || all
    }
  }

  pub fn model() -> AttributeArray
  {
    vec![(AttributeInfo::new(
      "print",
      "output the generated code on the standard output.",
      SubAttribute(vec![
        AttributeInfo::simple(
          "parser",
          "output the parser code."
        ),
        AttributeInfo::simple(
          "ast",
          "output the abstract syntax tree code."
        ),
        AttributeInfo::simple(
          "info",
          "output a header comment with the library version and license."
        ),
        AttributeInfo::simple(
          "code",
          "output all the code generated, equivalent to `#![print(ast, parser)]`."
        ),
        AttributeInfo::simple(
          "all",
          "output everything, equivalent to `#![print(code, info)]`."
        )
      ])
    ))]
  }
}
