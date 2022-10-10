#!/usr/bin/env python3
'''
read and compare two files with 10000 bits
'''

from sys import argv as args

if __name__ == "__main__":
    in_file, out_file = 'INPUT.txt', 'OUTPUT.txt'
    if len(args) > 1:
        in_file = args[1]
    if len(args) > 2:
        out_file = args[2]

    bits_in, bits_out = [], []

    with open(in_file, mode='r', encoding='utf-8') as f:
        bits_in = list(f.read().strip())
    with open(out_file, mode='r', encoding='utf-8') as f:
        bits_out = list(f.read().strip())

    for (i, (x, y)) in enumerate(zip(bits_in, bits_out)):
        if x != y:
            print(f'error on {i}: in={x} out={y}')
