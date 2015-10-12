// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub use self::rust::*;

grammar! rust {

  integer
    = decimal

  decimal = sign? number integer_suffix? > make_decimal

  sign
    = "-" > make_minus_sign
    / "+" > make_plus_sign

  number = digits > make_number
  digits = digit+ (underscore* digit)* > concat

  integer_suffix
    = "u8" > make_u8
    / "u16" > make_u16
    / "u32" > make_u32
    / "u64" > make_u64
    / "usize" > make_usize
    / "i8" > make_i8
    / "i16" > make_i16
    / "i32" > make_i32
    / "i64" > make_i64
    / "isize" > make_isize

  digit = ["0-9"]
  underscore = "_" -> (^)

  pub use syntax::ast::*;
  use std::str::FromStr;

  fn concat(mut x: Vec<char>, y: Vec<char>) -> Vec<char> {
    x.extend(y.into_iter());
    x
  }

  fn make_u8() -> LitIntType { UnsignedIntLit(TyU8) }
  fn make_u16() -> LitIntType { UnsignedIntLit(TyU16) }
  fn make_u32() -> LitIntType { UnsignedIntLit(TyU32) }
  fn make_u64() -> LitIntType { UnsignedIntLit(TyU64) }
  fn make_usize() -> LitIntType { UnsignedIntLit(TyUs) }
  fn make_i8() -> LitIntType { SignedIntLit(TyI8, Sign::Plus) }
  fn make_i16() -> LitIntType { SignedIntLit(TyI16, Sign::Plus) }
  fn make_i32() -> LitIntType { SignedIntLit(TyI32, Sign::Plus) }
  fn make_i64() -> LitIntType { SignedIntLit(TyI64, Sign::Plus) }
  fn make_isize() -> LitIntType { SignedIntLit(TyIs, Sign::Plus) }

  fn make_minus_sign() -> Sign { Sign::Minus }
  fn make_plus_sign() -> Sign { Sign::Plus }

  fn make_decimal(sign: Option<Sign>, number: u64, suffix: Option<LitIntType>) -> Lit_ {
    let sign = sign.unwrap_or(Sign::Plus);
    let ty = match suffix {
      None => UnsuffixedIntLit(sign),
      Some(SignedIntLit(ty, _)) => SignedIntLit(ty, sign),
      Some(UnsignedIntLit(_)) if sign == Sign::Minus => {
        panic!("unary negation of unsigned integers is forbidden.");
      },
      Some(x) => x
    };
    Lit_::LitInt(number, ty)
  }

  fn make_number(raw_number: Vec<char>) -> u64 {
    match u64::from_str(&*to_string(raw_number)).ok() {
      Some(x) => x,
      None => panic!("int literal is too large")
    }
  }

  fn to_string(raw_text: Vec<char>) -> String {
    raw_text.into_iter().collect()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use oak_runtime::*;

  #[test]
  fn integer_success_tests() {
    assert_eq!(parse_integer("123i32".stream()).unwrap_data(),
      Lit_::LitInt(123u64, LitIntType::SignedIntLit(IntTy::TyI32, Sign::Plus)));
    assert_eq!(parse_integer("-123i32".stream()).unwrap_data(),
      Lit_::LitInt(123u64, LitIntType::SignedIntLit(IntTy::TyI32, Sign::Minus)));
    // Overflows are checked by lints.
    assert_eq!(parse_integer("1000i8".stream()).unwrap_data(),
      Lit_::LitInt(1000u64, LitIntType::SignedIntLit(IntTy::TyI8, Sign::Plus)));
    assert_eq!(parse_integer("123_123_123i32".stream()).unwrap_data(),
      Lit_::LitInt(123123123u64, LitIntType::SignedIntLit(IntTy::TyI32, Sign::Plus)));
    assert_eq!(parse_integer("123".stream()).unwrap_data(),
      Lit_::LitInt(123u64, LitIntType::UnsuffixedIntLit(Sign::Plus)));
    assert_eq!(parse_integer("123u32".stream()).unwrap_data(),
      Lit_::LitInt(123u64, LitIntType::UnsignedIntLit(UintTy::TyU32)));
  }

  #[test]
  #[should_panic]
  fn too_large_integer() {
    parse_integer("10000000000000000000000".stream());
  }

  #[test]
  #[should_panic]
  fn negation_signed_integer() {
    parse_integer("-10000u8".stream());
  }
}
