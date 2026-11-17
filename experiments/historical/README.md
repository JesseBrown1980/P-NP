# Historical ASOLARIA Rime Fischer — byte-identical source archive

Original: https://github.com/JesseBrown1980/ASOLARIA-UNIVERSE-SIMULATOR-with-gravity-light-time-space-color-rime-winding-drills
Frozen upstream commit: 3f9f42dab6247c40b30921b253e3f73c9e942ca5.

Four files are copied unchanged from the original players/ directory and Git blob-checked:

| Historical file | Git blob SHA |
|---|---|
| players/rime_fischer.py | cbdf20564f1b38f3ddd08062ead8174ecc3d236a |
| players/rime_sphere.py | 449ac82b82223349ca8fc41d8e9b005fd3beda06 |
| players/rime_fischer_cluster.py | 2895e1e4a1fb4629005422429aeaf8020d15c6cf |
| players/rime_27fischers.py | c26228781c7e726b12efa8d18d6c68cc711cfd80 |

The historical original is Pohlig-Hellman prime-power digit lifting with Shanks baby-step/giant-step for order-q digits and exact CRT recombination. The historical helper rime_sphere.py supplies primality and primitive-root calculations. No content of these original four files has been edited.

## Hosted proof and comparison gates

1. Verify all four original Git blob hashes using git hash-object, independent of the test runners.
2. Execute the original rime_fischer.py unchanged as a script, preserving the five historical target results and checking their printed success status.
3. Execute historical clustered and 27-sphere variants unchanged and archive stdout/stderr; any exception is a test failure.
4. Run the original API on 21 identical discrete-log inputs, crosschecking original modular exponentiation and independently implemented full-order Shanks BSGS. Reject forged exponent and SHA receipts.
5. Run the current new algebra/rime_fischer.py and its independent verifier on exactly those same 21 inputs; cross-check matching exponents and preserve separate kernel timing records.
6. Archive the source files, exact outputs, SHA manifest and tests. A second hosted job downloads and verifies the published evidence independently.

## Scope

This is a third historical *mathematical implementation*, alongside the modern Rime code and the original Yang-Mills SAT solver. Discrete logarithm and exact SAT model counting are different problems. Direct mathematical comparisons do not establish SAT speedups or a theorem about P versus NP. An exact, independently justified reduction from SAT would be needed before running the original discrete-log implementation as a SAT solver.

To keep historical provenance intact, all adapters, comparisons and workflows live outside historical/. Future changes to an original must use a distinct versioned copy.