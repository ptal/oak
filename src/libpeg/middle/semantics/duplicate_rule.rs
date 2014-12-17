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

pub use rust::ExtCtxt;

use middle::semantics::ast::*;
use middle::semantics::duplicate::*;

pub fn rule_duplicate<'a>(cx: &'a ExtCtxt<'a>, grammar: Grammar,
  rules: Vec<Rule>) -> Partial<Grammar>
{
  DuplicateItem::analyse(cx, rules.into_iter(), String::from_str("rule"))
    .map(move |rules| grammar.with_rules(rules))
}
