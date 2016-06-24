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

pub struct Context<'a>
{
  grammar: &'a TGrammar<'a>,
  variables: Vec<Ident>,
  continuation_success: Box<CompileCombinator>,
  continuation_failure: Box<CompileCombinator>
}

pub trait CompileCombinator
{
  fn compile_combinator<'a>(self, context: Context<'a>) -> RExpr;
}


struct Sequence
{
  grammar: TGrammar,
  sequence: Vec<usize>
}

impl CompileCombinator for Sequence
{
  fn compile_combinator<'a>(self, mut success_cont: RExpr, failure_cont: RExpr, ident: &mut Vec<Ident>) -> RExpr {
    for idx in self.seq.rev() {
      success_cont = compile_combinator(idx, success_cont, failure_cont.clone(), ident);
    }
    success_cont
  }
}

struct Choice
{
  grammar: TGrammar,
  choice: Vec<usize>
}

impl CompileCombinator for Choice
{
  fn compile_combinator<'a>(self, mut success_cont: RExpr, failure_cont: RExpr, ident: &mut Vec<Ident>) -> RExpr {
    let mark = self.gen_mark_name();
    for idx in self.choice.rev() {
      failure_cont = quote_expr!(self.cx,
        state.restore($mark);
        $failure_cont
      );
      failure_cont = compile_combinator(idx, success_cont, failure_cont, &mut ident.clone());
    }
    quote_expr!(self.cx,
      let $mark = state.mark();
      $failure_cont
    )
  }
}
