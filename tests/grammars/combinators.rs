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

pub use self::combinators::*;

grammar! combinators {
  // #![debug_api]
  // #![show_api]

  str_literal = "return"

  sequence = "if" " " "then" " " "else"

  any_single_char = . .

  choice = "if" . "else" .
         / "let" . .
         / "if " ("-" . / .) " else " ("-" . / "+" .)

  repeat = (("a" / "b"+ ) .)* "c"*

  predicate = &"a" (!"b" .)+ / &"b" (!"a" .)+

  optional = "a"? "b" ("c" . / "d" .)? "z"

  char_class = ["a-zA-Z12_"]+ ["\t "]? ["-"]

  non_terminal = "a" non_terminal_bis+ .

  non_terminal_bis = ("b" . / "c" .) (!"d" .)+
}
