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

pub use self::stream_span::*;

grammar! stream_span {
  // #![debug_api]
  // #![show_api]

  // Optional stream declaration.
  type Stream<'a> = StrStream<'a>;

  expr = .. span_a . (.. .) "b" > make_expr

  span_a = .. "a"

  use oak_runtime::str_stream::*;

  pub struct Expr {
    pub full_sp: Span,
    pub span_a: Span,
    pub c2: char,
    pub c3_sp: Span,
    pub c3: char
  }

  fn make_expr(full_sp: Span, span_a: Span, c2: char, c3_sp: Span, c3: char) -> Expr {
    Expr {
      full_sp: full_sp,
      span_a: span_a,
      c2: c2,
      c3_sp: c3_sp,
      c3: c3
    }
  }
}

#[test]
fn test_stream_span() {
  use oak_runtime::*;

  let state = stream_span::parse_expr("abcb".into_state());
  let data = state.unwrap_data();
  assert_eq!(data.c2, 'b');
  assert_eq!(data.c3, 'c');
  assert_eq!(data.full_sp, make_span(0, 4));
  assert_eq!(data.span_a, make_span(0, 1));
  assert_eq!(data.c3_sp, make_span(2, 3));
}
