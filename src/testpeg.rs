#![feature(phase)]

#[phase(plugin)]
extern crate peg;

fn main() 
{
  let ntcc_grammar = peg!(
    ENTAIL = "|="
  );
}
