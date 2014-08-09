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

pub struct CodeGeneration
{
  pub ast: bool,
  pub parser: bool
}

impl Default for CodeGeneration
{
  fn default() -> CodeGeneration
  {
    CodeGeneration {
      ast: true,
      parser: true
    }
  }
}

// impl CodeGeneration
// {
//   pub fn register(attr_dict: &mut AttributeDict)
//   {
//     attr_dict.push(AttributeInfo::new(
//       "disable_code",
//       "the specified code won't be generated.",
//       SubAttribute(Rc::new(
//         AttributeDict::new(vec![
//             AttributeInfo::simple(
//               name: "parser",
//               desc: "do not generate the parser code."
//             ),
//             AttributeInfo::simple(
//               name: "ast",
//               desc: "do not generate the abstract syntax tree code."
//             )
//           ])
//         ))
//     ))
//   }
// }

// impl SetByName for CodeGeneration
// {
//   fn set_by_name<T>(&mut self, cx: &'a ExtCtxt, name: &str, value: &AttributeValue<T>)
//   {
//     if name == "ast" {
//       self.ast = value.value_or(self.ast);
//     } else if name == "parser" {
//       self.parser = value.value_or(self.parser);
//     } else {
//       default_set_by_name(cx, name, value);
//     }
//   }
// }

