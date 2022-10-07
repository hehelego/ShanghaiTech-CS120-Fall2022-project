#!/usr/bin/env python3
'''
generate 10000 random bits and write them into `input.txt`
'''

if __name__ == "__main__":
    import numpy as np
    rng = np.random.default_rng()
    bits = rng.choice([0, 1], 10000)
    with open('input.txt', mode='w', encoding='utf-8') as f:
        print(*bits, sep='', end='', file=f)
