# Attributions

## Turing machine algorithms

The arithmetic machines in `machines/` were written for this project, following these sources.

- `add`: ripple-carry addition, least significant bit first, with the carry kept in the state, as in the "Add two n-bit integers" algorithm of Boaz Barak, *Introduction to Theoretical Computer Science*, [§4.2 "Extended example: Addition and Multiplication"](https://introtcs.org/public/lec_03a_computing_every_function.html). It runs in linear time, unlike the increment/decrement loop of Wayne Eberly's [CPSC 351 Lecture #12 notes](https://cspages.ucalgary.ca/~eberly/Courses/CPSC351/2025/3_Turing_Machines/L12/L12_presentation_notes.pdf) (University of Calgary), which takes time exponential in the number of bits.
- `subtract`: column-by-column subtraction with borrows, right to left, from Wikipedia, ["Binary number", § Subtraction](https://en.wikipedia.org/wiki/Binary_number#Subtraction), truncated at 0 as in ["Monus"](https://en.wikipedia.org/wiki/Monus). Eberly's [CPSC 351 Tutorial #12](https://cspages.ucalgary.ca/~eberly/Courses/CPSC351/2025/3_Turing_Machines/T12/tutorial_12.pdf) builds the same function by repeated decrements, again in exponential time.
- `multiply`: grade-school (shift-and-add) multiplication, Algorithm 0.2 of Barak, [§0.1 "Integer multiplication: an example of an algorithm"](https://introtcs.org/public/lec_01_introduction.html), and Wikipedia, ["Multiplication algorithm", § Long multiplication](https://en.wikipedia.org/wiki/Multiplication_algorithm#Long_multiplication). It reads b from its most significant bit, Horner style (P ← 2P + b_j · a), in O(|b| · (|a| + |b|)) steps.
- Two inputs a and b are given as pf(NtS(a)) NtS(b), with the prefix-free encoding pf of Barak, [§2.5.2](https://introtcs.org/public/lec_02_representation.html).

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
