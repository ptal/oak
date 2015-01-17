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

use std::default::Default;
use attribute::model::*;
use attribute::model::AttributeModel::*;

pub struct CodeGeneration
{
  pub ast: bool,
  pub parser: bool
}

impl CodeGeneration
{
  pub fn new(model: &AttributeArray) -> CodeGeneration
  {
    let model = access::sub_model(model, "disable_code");
    CodeGeneration {
      ast: !access::plain_value_or(model, "ast", false),
      parser: !access::plain_value_or(model, "parser", false)
    }
  }

  pub fn model() -> AttributeArray
  {
    vec![AttributeInfo::new(
      "disable_code",
      "the specified code won't be generated.",
      SubAttribute(vec![
        AttributeInfo::simple(
          "parser",
          "do not generate the parser code."
        ),
        AttributeInfo::simple(
          "ast",
          "do not generate the abstract syntax tree code."
        )
      ])
    )]
  }
}
