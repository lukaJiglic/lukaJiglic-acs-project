"""Reference for the golden deal of spec 002, criterion 9.

A separate derivation of the deal for a seed, written from the steps in
src/engine/src/deal.rs. Pure Python: its own ChaCha8, no library and no
engine code. Standard library only; keep it that way, CI runs it with the
runner's python3.

It gives the two listings frozen in tests/deal.rs (seeds 0 and
81985529216486895). The workflow .github/workflows/engine.yml compares its
output with `cargo run --example print_deal` for those and three more seeds.

Usage from src/engine:   python tests/golden_deal_reference.py 0
Output: the four hands in index order, then the talon in the order it lies.
"""
import struct, sys

M = 0xFFFFFFFF
def rotl(x, n): return ((x << n) & M) | (x >> (32 - n))
def qr(s, a, b, c, d):
    s[a] = (s[a] + s[b]) & M; s[d] = rotl(s[d] ^ s[a], 16)
    s[c] = (s[c] + s[d]) & M; s[b] = rotl(s[b] ^ s[c], 12)
    s[a] = (s[a] + s[b]) & M; s[d] = rotl(s[d] ^ s[a], 8)
    s[c] = (s[c] + s[d]) & M; s[b] = rotl(s[b] ^ s[c], 7)

def chacha8_words(key):
    const = struct.unpack('<4I', b'expand 32-byte k')
    k = struct.unpack('<8I', key)
    counter = 0
    while True:
        init = list(const) + list(k) + [counter & M, counter >> 32, 0, 0]
        s = init[:]
        for _ in range(4):  # 8 rounds = 4 double rounds
            qr(s, 0, 4, 8, 12); qr(s, 1, 5, 9, 13); qr(s, 2, 6, 10, 14); qr(s, 3, 7, 11, 15)
            qr(s, 0, 5, 10, 15); qr(s, 1, 6, 11, 12); qr(s, 2, 7, 8, 13); qr(s, 3, 4, 9, 14)
        for i in range(16):
            yield (s[i] + init[i]) & M
        counter += 1

def deal(seed):
    words = chacha8_words(struct.pack('<Q', seed) + bytes(24))
    def below(n):
        skip = (2**32) % n
        while True:
            w = next(words)
            if w >= skip:
                return w % n
    deck = list(range(54))
    for i in range(53, 0, -1):
        j = below(i + 1)
        deck[i], deck[j] = deck[j], deck[i]
    talon = deck[:6]
    hands = [[], [], [], []]
    for p in range(8):
        hands[p % 4] += deck[6 + 6 * p: 12 + 6 * p]
    return [sorted(h) for h in hands], talon

hands, talon = deal(int(sys.argv[1]) if len(sys.argv) > 1 else 0)
for h in hands: print(h)
print(talon)
