#!/usr/bin/env python3
"""Diff two instruction traces with resynchronisation, and say which divergences matter.

    cargo run -p rom_test -- trace rom.nes --instructions 440000 > /tmp/ours.txt
    NESREF_TRACE=1 cargo run --manifest-path tools/nesref/Cargo.toml -- rom.nes 40 > /tmp/theirs.txt
    python3 tools/tracediff.py /tmp/ours.txt /tmp/theirs.txt

A plain `diff` stops at the first difference, which on a real ROM is nearly always a poll loop
spinning a different number of times — benign, and never what anyone is looking for. This walks
both streams and, on a mismatch, scans ahead in each for a window that matches again. Each
divergence becomes a block, and a block whose two sides are drawn from the same handful of
addresses is a loop counting differently rather than a different path through the program.

Two things it taught us on `5-branch_delays_irq`, worth knowing before using it again:

  - Align the two machines first. `RN_RESET_DOTS=19` puts ours where tetanes starts; without it the
    streams part company at the first vblank-boundary poll and everything after compares different
    sub-tests. It moved the first real divergence from instruction 42,443 to 175,788.
  - Read the *state* at the divergence, not only the addresses. The fault there was two
    `BIT $4015` reads straddling the frame counter's three-cycle IRQ window, which an address
    stream alone cannot show.
"""

import sys, collections

WINDOW = 8        # instructions that must match to call it resynchronised
LOOKAHEAD = 20000 # how far to search for that

def load(path, col=0):
    out=[]
    for line in open(path):
        if line.startswith('$'):
            out.append(line.split()[col])
    return out

def resync(a, b, i, j):
    """Find the nearest (di, dj) where a[i+di:] and b[j+dj:] match for WINDOW."""
    for total in range(1, LOOKAHEAD):
        for di in range(0, total+1):
            dj = total - di
            if i+di+WINDOW <= len(a) and j+dj+WINDOW <= len(b):
                if a[i+di:i+di+WINDOW] == b[j+dj:j+dj+WINDOW]:
                    return di, dj
    return None

def main(pa, pb):
    a, b = load(pa), load(pb)
    print(f"ours {len(a)} instructions, reference {len(b)}")
    i = j = 0
    blocks = 0
    while i < len(a) and j < len(b):
        if a[i] == b[j]:
            i += 1; j += 1
            continue
        found = resync(a, b, i, j)
        if not found:
            print(f"\n[{i}/{j}] diverged and never resynchronised")
            print(f"  ours      {' '.join(a[i:i+10])}")
            print(f"  reference {' '.join(b[j:j+10])}")
            return
        di, dj = found
        ours, theirs = a[i:i+di], b[j:j+dj]
        kind = "LOOP" if set(ours) <= set(ours+theirs) and set(ours) == set(theirs) or not ours or not theirs and False else "PATH"
        # A block is a loop difference if both sides are drawn from the same address set.
        same_set = set(ours) == set(theirs) or (not ours and set(theirs) <= set(b[max(0,j-12):j])) or (not theirs and set(ours) <= set(a[max(0,i-12):i]))
        kind = "loop " if same_set else "PATH "
        blocks += 1
        print(f"[{i}/{j}] {kind} ours +{di:5d}  reference +{dj:5d}   "
              f"ours{{{','.join(sorted(set(ours))[:4])}}} ref{{{','.join(sorted(set(theirs))[:4])}}}")
        if kind == "PATH ":
            print(f"        ours      {' '.join(a[i:i+8])}")
            print(f"        reference {' '.join(b[j:j+8])}")
            if blocks > 3: return
        i += di; j += dj
    print("streams ran out")

main(sys.argv[1], sys.argv[2])
