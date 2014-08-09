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

pub struct CodePrinter
{
  pub info: bool,
  pub ast: bool,
  pub parser: bool
}

impl CodePrinter
{
  pub fn new(model: &AttributeDict) -> CodePrinter
  {
    let model = model.sub_model("print");
    let ast = model.plain_value_or("ast", false);
    let parser = model.plain_value_or("parser", false);
    let info = model.plain_value_or("info", false);
    let code = model.plain_value_or("code", false);
    let all = model.plain_value_or("all", false);
    CodePrinter {
      ast: ast || code || all,
      parser: parser || code || all,
      info: info || all
    }
  }

  pub fn register(model: &mut AttributeDict)
  {
    model.push(AttributeInfo::new(
      "print",
      "output the generated code on the standard output.",
      SubAttribute(
        AttributeDict::new(vec![
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
        )
    ))
  }
}
