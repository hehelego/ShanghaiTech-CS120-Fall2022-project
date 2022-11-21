#!/usr/bin/env python3

import random
import sys


def gen(sz: int, name: str):
    with open(name, "wb") as f:
        f.write(random.randbytes(sz))


if len(sys.argv) > 1:
    random.seed(int(sys.argv[1]))

gen(6250, 'INPUT.bin')
gen(6250, 'INPUT1to2.bin')
gen(5000, 'INPUT2to1.bin')
