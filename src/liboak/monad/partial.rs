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

//! Partial is similar to Option where `Value` replaces `Some` and `Nothing` replaces `None`.
//!
//! `Fake` means that we got a value to pass for continuation (e.g. in `map` or `and_then`) but without real meaning, so it's an error to unwrap it.
//!
//! Value transformation are only from Value to Fake to Nothing which means that a Fake value will never be a Value again.
//! Use case: When compiling, an error in one function must be reported but should
//! not prevent the compilation of a second function to detect more errors in one run.
//! This intermediate state is represented by `Fake`.

use monad::partial::Partial::*;



#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Partial<T>
{
  Value(T),
  Fake(T),
  Nothing
}

impl<T> Partial<T>
{
  pub fn unwrap(self) -> T {
    match self {
      Value(x) => x,
      Fake(_) => panic!("called `Partial::unwrap()` on a `Fake` value"),
      Nothing => panic!("called `Partial::unwrap()` on a `Nothing` value")
    }
  }

  pub fn unwrap_or_else<F>(self, f: F) -> T where
   F: FnOnce() -> T
  {
    match self {
      Value(x) => x,
      _ => f()
    }
  }

  pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Partial<U> {
    match self {
      Value(x) => Value(f(x)),
      Fake(x) => Fake(f(x)),
      Nothing => Nothing
    }
  }

  pub fn and_then<U, F: FnOnce(T) -> Partial<U>>(self, f: F) -> Partial<U> {
    match self {
      Value(x) => f(x),
      Fake(x) => match f(x) {
        Value(x) => Fake(x),
        x => x
      },
      Nothing => Nothing
    }
  }

  pub fn and_next<U, F: FnOnce(T) -> Partial<U>>(self, f: F) -> Partial<U> {
    match self {
      Value(x) => f(x),
      _ => Nothing
    }
  }
}

fn nothing_i32(_: i32) -> Partial<i32> {
  Nothing
}

#[test]
fn partial_unwrap() {
  assert_eq!(Value(9i32).unwrap(), 9i32);
}

#[test]
#[should_panic]
fn partial_unwrap_fake() {
  Fake(9i32).unwrap();
}

#[test]
#[should_panic]
fn partial_unwrap_nothing() {
  let x: Partial<i32> = Nothing;
  x.unwrap();
}

#[test]
fn partial_unwrap_or_else() {
  assert_eq!(Value(9i32).unwrap_or_else(|| 1i32), 9i32);
  assert_eq!(Fake(9i32).unwrap_or_else(|| 1i32), 1i32);
  assert_eq!(Nothing.unwrap_or_else(|| 1i32), 1i32);
}

#[test]
fn partial_map() {
  assert_eq!(Value(9i32).map(|i|i*2), Value(18i32));
  assert_eq!(Fake(9i32).map(|i|i*2), Fake(18i32));
  assert_eq!(Nothing.map(|i:i32|i), Nothing);
}

#[test]
fn partial_and_then() {
  assert_eq!(Value(9i32).and_then(|i| Value(i*2)), Value(18i32));
  assert_eq!(Value(9i32).and_then(|i| Fake(i*2)), Fake(18i32));
  assert_eq!(Fake(9i32).and_then(|i| Fake(i*2)), Fake(18i32));
  // Even if you return a Value, it automatically coerces to Fake.
  assert_eq!(Fake(9i32).and_then(|i| Value(i*2)), Fake(18i32));
  assert_eq!(Fake(9i32).and_then(nothing_i32), Nothing);
}

#[test]
fn partial_and_next() {
  assert_eq!(Value(1i32).and_next(|i| Value(i*2)), Value(2));
  assert_eq!(Value(1i32).and_next(|i| Fake(i*2)), Fake(2));
  assert_eq!(Value(1i32).and_next(nothing_i32), Nothing);
  assert_eq!(Fake(1i32).and_next(|i| Value(i*2)), Nothing);
  assert_eq!(nothing_i32(0).and_next(|i| Value(i*2)), Nothing);
}
