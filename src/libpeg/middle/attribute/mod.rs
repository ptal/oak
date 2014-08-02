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

pub use middle::attribute::code_printer::{CodePrinterBuilder, CodePrinter};
pub use middle::attribute::code_gen::{CodeGenerationBuilder, CodeGeneration};
pub use middle::attribute::start_rule::StartRuleBuilder;
pub use middle::attribute::rule_type::{RuleTypeBuilder, RuleType};

mod attribute;
mod code_printer;
mod code_gen;
mod start_rule;
mod rule_type;
mod single_attribute_builder;
