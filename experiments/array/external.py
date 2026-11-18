#!/usr/bin/env python3
"""Run unchanged external-arm decision and proof protocol on array-v1 input.

The SAT solvers have 12-second per-instance ceilings and CaDiCaL
DRAT-trim has a separate 12-second checker ceiling.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "experiments" / "hardest"))
import external_arm
import suite

external_arm.LIMIT_SECONDS = 12
external_arm.CERT_LIMIT_SECONDS = 12
external_arm.challenge.original = lambda case: suite.definition(case)[:2]


if __name__ == "__main__":
    assert len(sys.argv) == 6
    arm, case, source, archive, checker = sys.argv[1:]
    assert arm in suite.ARMS[3:] and case in suite.CASES
    external_arm.run_external(arm, case, Path(source), Path(archive),
                              Path(checker))
