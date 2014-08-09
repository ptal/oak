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

pub struct CodePrinter
{
  pub info: bool,
  pub ast: bool,
  pub parser: bool
}

impl Default for CodePrinter
{
  fn default() -> CodePrinter
  {
    CodePrinter {
      info: false,
      ast: false,
      parser: false
    }
  }
}

// impl CodePrinter
// {
//   pub fn register(attr_dict: &mut AttributeDict)
//   {
//     attr_dict.push(AttributeInfo::new(
//       "print",
//       "output the generated code on the standard output.",
//       SubAttribute(Rc::new(
//         AttributeDict::new(vec![
//             AttributeInfo::simple(
//               name: "parser",
//               desc: "output the parser code."
//             ),
//             AttributeInfo::simple(
//               name: "ast",
//               desc: "output the abstract syntax tree code."
//             ),
//             AttributeInfo::simple(
//               name: "info",
//               desc: "output a header comment with the library version and license."
//             ),
//             AttributeInfo::simple(
//               name: "code",
//               desc: "output all the code generated, equivalent to `#![print(ast, parser)]`."
//             ),
//             AttributeInfo::simple(
//               name: "all",
//               desc: "output everything, equivalent to `#![print(code, info)]`."
//             )
//           ])
//         ))
//     ))
//   }
// }

// impl SetByName for CodePrinter
// {
//   fn set_by_name<T>(&mut self, cx: &'a ExtCtxt, name: &str, value: &AttributeValue<T>)
//   {
//     if name == "info" {
//       self.info = value.value_or(self.info);
//     } else if name == "ast" {
//       self.ast = value.value_or(self.ast);
//     } else if name == "parser" {
//       self.parser = value.value_or(self.parser);
//     } else if name == "code" {
//       self.ast = value.value_or(self.ast);
//       self.parser = value.value_or(self.parser);
//     } else if name == "all" {
//       self.ast = value.value_or(self.ast);
//       self.parser = value.value_or(self.parser);
//       self.info = value.value_or(self.info);
//     } else {
//       default_set_by_name(cx, name, value);
//     }
//   }
// }
