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

//! This module decorates the grammar with the attributes attached to the items and gives them a semantics. It also checks for duplicate or unknown attributes.
//!
//! It depends on an external library [attribute](attribute/index.html) because an attribute is just a tree that must be model checked. We would like to extend this external library to any tree-shaped structure.

pub mod attribute;
pub mod ast;
pub mod visitor;
mod code_printer;
mod code_gen;
mod rule_type;
