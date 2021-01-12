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

use oak::oak;

oak! {
  // Optional stream declaration.
  type Stream<'a> = StrStream<'a>;

  expr = .. span_a . (.. .) "b" > make_expr

  expr2 = .. span_a . inner_expr "b" > make_expr
  inner_expr = (.. (./.))

  span_a = .. "a"

  expr3 = .. ("a" / "b") "c"

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

use oak_runtime::*;

fn test_state<'a>(state: ParseState<StrStream<'a>, Expr>)
{
  let data = state.unwrap_data();
  assert_eq!(data.c2, 'b');
  assert_eq!(data.c3, 'c');
  assert_eq!(data.full_sp, make_span(0, 4));
  assert_eq!(data.span_a, make_span(0, 1));
  assert_eq!(data.c3_sp, make_span(2, 3));
}

fn test_state_expr3<'a>(state: ParseState<StrStream<'a>, Span>) {
  assert_eq!(state.unwrap_data(), make_span(0,2))
}

#[test]
fn test_stream_span() {
  let state = parse_expr("abcb".into_state());
  let state2 = parse_expr2("abcb".into_state());
  let state3 = parse_expr3("ac".into_state());
  let state4 = parse_expr3("bc".into_state());
  test_state(state);
  test_state(state2);
  test_state_expr3(state3);
  test_state_expr3(state4);
}
