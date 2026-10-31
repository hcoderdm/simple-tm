# Attributions

## Turing machine algorithms

The machines in `machines/` were written for this project. Their algorithms come from these sources.

| Machine | Algorithm | Sources |
|---|---|---|
| `add` | Ripple-carry addition | Barak, *Introduction to Theoretical Computer Science*, [§4.2](https://introtcs.org/public/lec_03a_computing_every_function.html) |
| `subtract` | Subtraction with borrows, truncated at 0 | Wikipedia, ["Binary number"](https://en.wikipedia.org/wiki/Binary_number#Subtraction), ["Monus"](https://en.wikipedia.org/wiki/Monus) |
| `multiply` | Shift-and-add (long) multiplication | Barak, [§0.1](https://introtcs.org/public/lec_01_introduction.html); Wikipedia, ["Multiplication algorithm"](https://en.wikipedia.org/wiki/Multiplication_algorithm#Long_multiplication) |
| `divide`, `mod` | Binary long division | Wikipedia, ["Division algorithm"](https://en.wikipedia.org/wiki/Division_algorithm#Integer_division_(unsigned)_with_remainder) |
| `compare`, `equal` | Copy to a second tape and compare | Rajaraman, [CS G714 notes "7. Time Complexity"](https://www.ccs.neu.edu/home/rraj/Courses/G714/S09/Notes/7.TimeComplexity.txt); Wikipedia, ["Digital comparator"](https://en.wikipedia.org/wiki/Digital_comparator) |
| `increment` | Carry from the least significant bit | Barak, [Chapter 7](https://introtcs.org/public/lec_06_loops.html) |
| `palindrome` | Copy to a second tape and compare in reverse | Barak, [§7.1.1](https://introtcs.org/public/lec_06_loops.html) |
| `and`, `or`, `not`, `nand`, `nor`, `xor`, `xnor` | Bitwise logic gates | Wikipedia, ["Logic gate"](https://en.wikipedia.org/wiki/Logic_gate), ["Bitwise operation"](https://en.wikipedia.org/wiki/Bitwise_operation); Barak, [Chapter 3](https://introtcs.org/public/lec_03_computation.html) |
| Inputs of several numbers | Prefix-free encoding | Barak, [§2.5.2](https://introtcs.org/public/lec_02_representation.html) |

## Open Color

The colors in `python/window.py` come from [Open Color](https://github.com/yeun/open-color) by heeyeun, used under the MIT License:

```
MIT License

Copyright (c) 2016 heeyeun

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
