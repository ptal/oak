% Related Work

### Conventional parser generator

In traditional parser generators, the bridge between purely syntactic analysis and AST construction is done with semantic actions. Interaction between an expression and the user code is usually done with one of these two techniques (`digit` being a rule parsing an integer):

1. Positional arguments: `digit "+" digit { $$ = $1 + $3; }` is a technique used in [Yacc](http://dinosaur.compilertools.net/yacc/) for example.
2. Expression labelling: `digit:x "+" digit:y { x + y }` is similar to what is used in [Menhir](http://gallium.inria.fr/~fpottier/menhir/) (parser generator written in OCaml).

The first technique is often discouraged because some errors can silently appear if you change the order of expression inside a rule without changing the associated action or if you make a mistake when numbering the arguments. The generated code will fail to compile if the host language is statically typed and if the two expressions have different types, but in the general case this technique is not safe. Expression labelling is better but it has the inconvenient of burdening the grammar syntax. Also note that none of these techniques help the user to build the corresponding AST, their purposes is to offer a simple interface between grammar and host code.

Using the idea of typing grammar, we can give a type to each expression and directly pass the value to the semantic action without any labelling or positional notation. The previous example becomes `digit "+" digit > add` with `>` being a "reverse function call operator", the expression `digit "+" digit` produces a value `v` of type `(i32, i32)` and the code generated looks like `add(v)`. It is even smarter and will automatically unpack the tuple into function arguments, so the function `add` will be called with two arguments of type `i32`.

### Parser combinators

### Implementations

I read, get inspired or used some ideas of the following implementations (non-exhaustive list):

* [rust-peg](https://github.com/kevinmehall/rust-peg)
* [nom](https://github.com/Geal/nom)
* [combine](https://github.com/Marwes/combine)
* [Pegged](https://github.com/PhilippeSigaud/Pegged): Annotations in expressions for dropping, discarding, keeping or fusing AST nodes.
* [Rats!](https://cs.nyu.edu/rgrimm/xtc/rats-intro.html)
* [Mouse](http://www.romanredz.se/freesoft.htm)
* [Boost.Spirit](http://www.boost.org/doc/libs/1_59_0/libs/spirit/doc/html/index.html): It takes the approach of inferring a type and try to make it compatible with the type provided by the user.
* [pegjs](http://pegjs.org/)

### Paper references

* The initial article of Brian Ford is at the heart of Oak. Bryan Ford. [Parsing expression grammars: a recognition-based syntactic foundation](http://www.bford.info/pub/lang/peg.pdf). In ACM SIGPLAN Notices, volume 39, pages 111–122. ACM, 2004.
* The following article helped me for error reporting, however there is still more to get from it. André Murbach Maidl, Sérgio Medeiros, Fabio Mascarenhas, and Roberto Ierusalimschy. [Error reporting in parsing expression grammars](http://arxiv.org/abs/1405.6646). arXiv preprint arXiv:1405.6646, 2014.
* Robert Grimm. [Better extensibility through modular syntax](http://cs.nyu.edu/rgrimm/papers/pldi06.pdf). In ACM SIGPLAN Notices, volume 41, pages 38–51. ACM, 2006.
* ...
