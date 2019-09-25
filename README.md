# CPEN 442 Analyzer

This project contains 3 binaries for CPEN 442 Assignment 2.

## P1

This is does simulated annealing to solve a substitution cipher stored in `cipher.txt`.

```sh
$ cargo rustc --bin p1 --release -- -C target-cpu=native
$ target/release/p1
```

## P2

This is does simulated annealing to solve a Playfair cipher stored in `cipher2.txt`.

```sh
$ cargo rustc --bin p2 --release -- -C target-cpu=native
$ target/release/p2
```

## P1 P2 Data Files

Both P1 and P2 use these data files for simulated annealing

**english_bigrams.txt**

Common english bigrams and how frequently they appear.

Currently not used as part of the simulated annealing.

**english_quadgrams.txt**

Common english quadgrams and how frequently they appear.

This is the main data used for simulated annealing

**wordlist.txt**

1000 of the most common english words of various length.

This is mainly used to provide a "word coverage" statistic on deciphered text.


## P3 and P4

P3 and P4 share the same executable.

This executable checks many random ASCII strings for colliding crcs.

When provided with a argument it will find a string with the same crc as the argument.

```sh
$ cargo rustc --bin p3 --release -- -C target-cpu=native

# Strong collision: Find any two strings with same crc32
$ target/release/p3

# Weak collision: Find a string with the same crc as abcd
$ target/release/p3 abcd
```
