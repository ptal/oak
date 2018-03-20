#![feature(plugin)]
#![plugin(oak)]

extern crate oak_runtime;

grammar! useless_chaining {
  #![show_api]
  test1 = !(!"a") // &"a"
  test2 = &(&"a") // &"a"
  test3 = !(&"a") // !"a"
  test4 = &(!"a") // !"a"
  //test5 = ("a"*)* // infinite loop -> deja detectee
  test6 = ("a"+)+ // "a"+
  test7 = ("a"+)* // "a"+
  //test8 = ("a"*)+ // infinite loop -> deja detectee

  test9 = !"a"
  test10 = !test9

  test11 = &"a"
  test12 = &test11

  test13 = !test11

  test14 = &test9

  test15 = "a"+
  test16 = test15+

  test17 = test15*

  test18 = &test12
  test19 = test16+

  test20 = ((("a")+)+)+
  test21 = &(&(&("a")))
}

fn main() {
  println!("test");
}
