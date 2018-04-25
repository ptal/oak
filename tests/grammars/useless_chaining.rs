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

pub use self::useless_chaining::*;

grammar! useless_chaining {

  // test1 = !(!"a") // &"a"
  // test2 = &(&"a") // &"a"
  // test3 = !(&"a") // !"a"
  // test4 = &(!"a") // !"a"
  // //test5 = ("a"*)* // infinite loop -> deja detectee
  // test6 = ("a"+)+ // "a"+
  // test7 = ("a"+)* // "a"+
  // //test8 = ("a"*)+ // infinite loop -> deja detectee
  //
  test9 = !"a"
  // test10 = !test9
  //
  // test11 = &"a"
  // test12 = &test11
  //
  // test13 = !test11
  //
  // test14 = &test9
  //
  // test15 = "a"+
  // test16 = test15+
  //
  // test17 = test15*
  //
  // test18 = &test12
  // test19 = test16+
  //
  // test20 = ((("a")+)+)+
  // test21 = &(&(&(&("a"))))
  //
  // test22 = &"a" / !"b"
  // test23 = &test22
  //
  // test24 = &"a" "b"
  // test25 = &test24
  //
  // test26 = &"a" / &"b"
  // test27 = &test26
  //
  // test28 = &"a" / "b"
  // test29 = &test28
  //
  // test30 = &"a" &(!"b")
  // test31 = &test30
  //
  // test32 = &(&"a") / &(&"b")
  // test33 = &test32
}
