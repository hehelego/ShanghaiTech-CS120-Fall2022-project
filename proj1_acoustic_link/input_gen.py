#!/usr/bin/env python3

import random
import sys
k = 10000

if len(sys.argv) > 1:
    random.seed(int(sys.argv[1]))

print("".join([str(random.randint(0, 1)) for _ in range(k)]))
