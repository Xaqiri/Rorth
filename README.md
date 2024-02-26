# Rorth

A [stack-based](https://en.wikipedia.org/wiki/Stack-oriented_programming) [concatenative](https://en.wikipedia.org/wiki/Concatenative_programming_language) language written in Rust, using [Qbe](https://c9x.me/compile/) as the backend

Inspired by [Porth](https://gitlab.com/tsoding/porth) and [Forth](https://en.wikipedia.org/wiki/Forth_(programming_language)), though the design(and name) are likely to change in the future

## Examples

### Factorial
```forth
10 1 > while
  over * 
  swap 1 - swap
end .

\ with variables
x 1 :=
10 1 > while
  over x over * := drop
  swap 1 - swap
end x .
```

