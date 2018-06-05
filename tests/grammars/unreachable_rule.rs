// Copyright 2018 Chao Lin & William Sergeant (Sorbonne University)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub use self::unreachable_rule::*;

grammar! unreachable_rule{
    // test0 =  "a"*
    //         /"a"+  // is detected

    // test1 =  "a"
    //         /"ab"
    //
    test2 =  "ab"
            / "a"
    // r1 = ["a-c"] -> () / "b" -> () // problem of span
    //
    // test3 =  "a"
    //         /"a"
    //
    // test4 =  "abcd"
    //         /"a" "bc"
    //
    // test4bis = "a" "bc"
    //          / "abcd"

    // keyword = "proc"/"par"/"space"/"end"/"pre"
    //      / "read" / "write" / "readwrite" / "or" / "and" / "not"
    //      / "when" / "then" / "else" / "loop" / "pause up"
    //      / "pause" / "stop" / "in" / "word_line" / "singe_time"
    //      / "single_space" / "bot" / "top" / "ref" / "module"
    //      / "run" / "true" / "false" / "unknown" / "nothing"
    //      / "universe" / "suspend" / "abort" / "java_kw"

    // test5 =  "a"*
    //         /"a"?  // is detected

    // test6 =  "a"
    //         /!"a"
    //
    // test7 = !"a"
    //         /"a"
    //
    // test8 =  "a" !"a"
    //         /"a"
    //
    // test9 = "a"
    //         /&"a"
    //
    // test10 = &"a"
    //         /"a"
    //
    // test11 = "b"
    //         /!"a"

    // test12 = .
    //         /"a" // is detected
}
