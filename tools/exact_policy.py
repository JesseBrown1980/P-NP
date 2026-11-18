#!/usr/bin/env python3
"""Minimal exact-arithmetic source gate for this standalone repository.

Tests the tracked Python files' AST. This static audit is one acceptance
control, paired with independent proof replay and runtime exactness tests.
"""
import ast
import sys
from pathlib import Path

DENIED_IMPORTS = {
    "numpy", "scipy", "mpmath", "decimal", "fractions", "sympy",
    "statistics", "cmath", "timeit", "ctypes", "importlib",
    "pickle", "marshal", "json", "torch", "pandas",
}
DENIED_CALLS = {
    "float", "complex", "eval", "exec", "compile", "__import__",
}
DENIED_ATTRS = {
    "random", "uniform", "gauss", "sqrt", "pow",
    "getmtime", "total_seconds", "timestamp", "as_integer_ratio",
}


def is_path_expression(node):
    """Recognize a chained pathlib slash rather than numeric division."""
    if isinstance(node, ast.Name):
        return node.id in (
            "root", "ROOT", "path", "p", "base", "folder", "archive",
            "evidence", "logs", "location", "cnf", "source_path",
            "stdout_path", "stderr_path", "memory_path")
    if isinstance(node, ast.Attribute) and node.attr == "parent":
        return is_path_expression(node.value)
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div):
        return is_path_expression(node.left)
    return False


def audit(path):
    source = Path(path).read_text(encoding="utf-8")
    root = ast.parse(source, filename=str(path))
    issues = []
    for node in ast.walk(root):
        if isinstance(node, ast.Constant) and isinstance(node.value, (float, complex)):
            issues.append(("noninteger_literal", node.lineno))
        elif isinstance(node, ast.BinOp) and (
                isinstance(node.op, ast.Pow) or
                (isinstance(node.op, ast.Div) and
                 not is_path_expression(node.left))):
            issues.append(("potential_noninteger_operation", node.lineno))
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            modules = ([alias.name for alias in node.names]
                       if isinstance(node, ast.Import)
                       else [node.module or ""])
            for name in modules:
                if name.split(".")[0] in DENIED_IMPORTS:
                    issues.append(("noninteger_or_dynamic_import", node.lineno))
        elif isinstance(node, ast.Call):
            fn = node.func
            if isinstance(fn, ast.Name) and fn.id in DENIED_CALLS:
                issues.append(("dynamic_or_noninteger_call", node.lineno))
            elif isinstance(fn, ast.Attribute) and fn.attr in DENIED_ATTRS:
                issues.append(("noninteger_method", node.lineno))
    return issues


def main(paths):
    assert paths
    fail = 0
    for path in paths:
        findings = audit(path)
        fail += len(findings)
        print("PNP_EXACT_AUDIT|file=" + str(path) + "|findings=" +
              str(len(findings)) + "|status=" + ("HOLD" if findings else "PASS") +
              "|json=0")
        for kind, line in findings:
            print("PNP_EXACT_FINDING|file=" + str(path) +
                  "|line=" + str(line) + "|kind=" + kind + "|json=0")
    assert fail == 0


if __name__ == "__main__":
    main(sys.argv[1:])
