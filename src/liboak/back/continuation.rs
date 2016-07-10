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

pub use middle::typing::ast::*;
use back::compiler::ExprCompilerFn;
use back::context::Context;

pub struct Continuation
{
  success: RExpr,
  failure: RExpr
}

impl Continuation
{
  pub fn new(success: RExpr, failure: RExpr) -> Self {
    Continuation {
      success: success,
      failure: failure
    }
  }

  pub fn compile_success(self, context: &mut Context,
    compiler: ExprCompilerFn, idx: usize) -> Self
  {
    self.map_success(|success, failure|
      context.compile_success(compiler, idx, success, failure))
  }

  pub fn compile_and_wrap(&self, context: &mut Context,
    compiler: ExprCompilerFn, idx: usize, before_success: RStmt) -> RExpr
  {
    let cx = context.cx();
    let success = self.success.clone();
    context.compile_success(compiler, idx,
      quote_expr!(cx, {
        $before_success
        $success
      }),
      self.failure.clone())
  }

  pub fn map_success<F>(mut self, f: F) -> Self where
   F: FnOnce(RExpr, RExpr) -> RExpr
  {
    self.success = f(self.success, self.failure.clone());
    self
  }

  pub fn unwrap_success(self) -> RExpr {
    self.success
  }

  pub fn unwrap(self) -> (RExpr, RExpr) {
    (self.success, self.failure)
  }
}
