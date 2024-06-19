````fn something(in1: bool) -> uint {
    let a = 3
    let b = 4
    if a > 3 {
       return b
    } else {
      return 10
    }
}
````

something(bool):int
  1 const 3
  2 store 2
  3 const 4
  4 store 3
  5 const 3
  6 load 2
  7 ileq 10
  8 load 3
  9 return
  10 const 10
  11 return
