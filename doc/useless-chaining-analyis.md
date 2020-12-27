# Oak - Rust Module - Useless chaining detection / Non-reachable expressions in prioritized choice
###### [Original repository of Oak by ptal](https://github.com/ptal/oak "Github ptal")
#

# Useless chaining detection

***
Most chains will be forbidden by the well-formed analysis. I think that other chains are not incorrect (&&e or e+* for example) but are just bad style or weird. IMHO, a syntactic analysis is enough if we restrict only one prefix and suffix for a single expression. For prefixes, !e is like not e in logic and &e like e, indeed & is just a syntactic sugar for !!e. Thus we can see & as a hint to the parser generator to avoid consuming input, it is an identity function with regards to the result of e. Since this hint is already provided by !e, chaining & and ! is useless. We have these equivalences:

```javascript
!!e ~ &e
&&e ~ &e
!&e ~ !e
&!e ~ !e
```

Let's write down the cases for suffixes:

```javascript
e** ~ infinite loop
e++ ~ e+
e+* ~ e+
e*+ ~ infinite loop
```

The aim is to analyze AST in order to catch useless chaining occurences by identiying nodes consituting unoptimized chains.
We use visitors to identify each AST node and mark chains such as & ! + * and fill a vector of following characters then reducing it to give developpers advice about potential optimizations in their rule definition.
