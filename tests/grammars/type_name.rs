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

pub use self::type_name::*;

grammar! type_name{
  // #![debug_api]

  lparen = "(" spacing

  type_name = auto_infer_kw &(lparen / not_eof / comma)
            / ident

  type_names = spacing type_name (lparen type_names (comma type_names)* rparen)?

  spacing = [" \n\t"]* -> (^)

  ident = !["0-9"] ["a-zA-Z0-9_"]+ spacing -> (^)
  auto_infer_kw = "_" spacing
  rparen = ")" spacing
  not_eof = !.
  comma = "," spacing
}
