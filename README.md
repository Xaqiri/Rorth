# Rorth

A [stack-based](https://en.wikipedia.org/wiki/Stack-oriented_programming) [concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language) language written in Rust, using [Qbe](https://c9x.me/compile/) as the backend

Inspired by [Porth](https://gitlab.com/tsoding/porth) and [Forth](https://en.wikipedia.org/wiki/Forth_(programming_language)), though the design(and name) are likely to change in the future

## Examples

### Conditionals
```forth
1 1 = if
  1 .
else 
  0 .
end 
```

### Loops
```forth
10 1 > while
  over . 
  swap 1 - swap
end .
```

### Factorial
```forth
1 x :=
10 1 > while
  over x * :=
  swap 1 - swap
end x .
```

### Custom Words
```forth
: fib (a -- a) 
1 x := 
1 > while 
  over x * := 
  swap 1 - swap 
end x . ; 

10 fib \ 3628800
5 fib \ 120
```

