#!/usr/bin/env python3
"""Per-test mechanical inventory of every #[test] fn in a quanta-gqlang-v2 checkout.

Usage:
    python3 inventory_scan.py <repo-root> [out.tsv]

Emits one row per test function: file, line, test name, assertion count, body
size, mechanical smell flags. Intended to be re-run whenever the tree moves, so
the shipped test-suite-inventory.tsv never silently goes stale.

Flag vocabulary (all are candidates for human judgement, not verdicts):
  NO_ASSERTION          body contains no assert*/panic!/unwrap_err/expect_err
  UNWRAP_AS_ONLY_CHECK  unwrap/expect present with <=1 real assertion
  NO_EXACT_ORACLE       no assert_eq!/assert_ne!/panic!/unwrap_err anywhere
  ONLY_WEAK_PREDICATE   every assertion is a weak predicate (is_err/is_ok/...)
  MATCHES_IGNORE_PAYLOAD matches!(.., X { .. }) discards the payload
  TAUT_OK_OR_ERR        assert!(r.is_ok() || r.is_err())
  SELF_COMPARE_EQ/NE    assert_eq!(X, X) with identical operands
  EARLY_RETURN_SKIP     unconditional `return;` in the body
  LET_UNDERSCORE        `let _ =` in the body
  PROPTEST              body is inside a proptest! block
"""
import os, re, sys, hashlib
from collections import Counter, defaultdict

TESTATTR = re.compile(r'#\[(test|tokio::test|rstest|test_case)[^\]]*\]')
ASSERT_RE = re.compile(
    r'\bassert(?:_eq|_ne)?!\s*\(|\bpanic!\s*\(|\bunreachable!\s*\('
    r'|\bexpect_err\s*\(|\bunwrap_err\s*\('
    r'|\bprop_assert(?:_eq|_ne)?!\s*\(|\bassert_matches!\s*\(')


def _blank(ch):
    return '\n' if ch == '\n' else ' '


def strip_strings_comments(s):
    """Length-preserving blanking of comments and string/char literals."""
    out = list(s)
    i, n = 0, len(s)
    while i < n:
        c = s[i]
        if c == '/' and i + 1 < n and s[i + 1] == '/':
            while i < n and s[i] != '\n':
                out[i] = ' '
                i += 1
        elif c == '/' and i + 1 < n and s[i + 1] == '*':
            while i < n:
                if s[i] == '*' and i + 1 < n and s[i + 1] == '/':
                    out[i] = out[i + 1] = ' '
                    i += 2
                    break
                out[i] = _blank(s[i])
                i += 1
        elif c == '"':
            out[i] = ' '
            i += 1
            while i < n:
                if s[i] == '\\':
                    out[i] = out[i + 1] = ' '
                    i += 2
                    continue
                if s[i] == '"':
                    out[i] = ' '
                    i += 1
                    break
                out[i] = _blank(s[i])
                i += 1
        elif c == "'" and i + 2 < n and (
                s[i + 2] == "'" or (s[i + 1] == '\\' and i + 3 < n and s[i + 3] == "'")):
            end = i + 4 if s[i + 1] == '\\' else i + 3
            for k in range(i, min(end, n)):
                out[k] = _blank(s[k])
            i = end
        else:
            i += 1
    return ''.join(out)


def find_fns(src, code):
    for m in TESTATTR.finditer(code):
        fm = re.search(r'\bfn\s+([A-Za-z0-9_]+)', code[m.end():m.end() + 400])
        if not fm:
            continue
        fnpos = m.end() + fm.start()
        brace = code.find('{', fnpos)
        if brace < 0:
            continue
        depth, i, n = 0, brace, len(code)
        while i < n:
            if code[i] == '{':
                depth += 1
            elif code[i] == '}':
                depth -= 1
                if depth == 0:
                    break
            i += 1
        yield (src[:m.start()].count('\n') + 1, fm.group(1),
               src[brace:i + 1])


def classify(body):
    flags = []
    n_assert = len(ASSERT_RE.findall(body))
    if re.search(r'\.is_ok\(\)\s*\|\|\s*\w+\.is_err\(\)', body) or \
       re.search(r'\.is_err\(\)\s*\|\|\s*\w+\.is_ok\(\)', body):
        flags.append('TAUT_OK_OR_ERR')
    for name, pat in (('SELF_COMPARE_EQ', r'assert_eq!'), ('SELF_COMPARE_NE', r'assert_ne!')):
        for m in re.finditer(pat + r'\s*\(\s*([^,()]+?)\s*,\s*([^,()]+?)\s*[,)]', body):
            a = re.sub(r'\s+', '', m.group(1))
            c = re.sub(r'\s+', '', m.group(2))
            if a == c and len(a) > 2:
                flags.append(name)
                break
    weak = len(re.findall(r'assert!\(\s*[\w.()\[\]:&* ]*\.is_(err|ok|some|none)\(\)', body))
    if n_assert and weak == n_assert:
        flags.append('ONLY_WEAK_PREDICATE')
    if weak and not re.search(r'\bassert_eq!|\bassert_ne!|\bpanic!|\bexpect_err|\bunwrap_err', body):
        flags.append('NO_EXACT_ORACLE')
    if re.search(r'matches!\([^;]*?,\s*_?\w*(::\w+)?\s*\{\s*\.\.\s*\}\s*\)', body):
        flags.append('MATCHES_IGNORE_PAYLOAD')
    if not re.search(r'\bassert|\bpanic!|\bexpect_err|\bunwrap_err', body):
        flags.append('NO_ASSERTION')
    if re.search(r'\blet\s+_\s*=', body):
        flags.append('LET_UNDERSCORE')
    if re.search(r'^\s*return\s*;', body, re.M):
        flags.append('EARLY_RETURN_SKIP')
    if re.search(r'\.unwrap\(\)|\.expect\(', body) and n_assert <= 1:
        flags.append('UNWRAP_AS_ONLY_CHECK')
    if re.search(r'\bproptest!|\bprop_assert', body):
        flags.append('PROPTEST')
    return flags, n_assert


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    root = os.path.abspath(sys.argv[1])
    out_path = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.getcwd(), 'test-suite-inventory.tsv')
    rows = []
    for dirpath, dirnames, filenames in os.walk(os.path.join(root, 'crates')):
        dirnames[:] = [d for d in dirnames if d != 'target']
        for fn in sorted(filenames):
            if not fn.endswith('.rs'):
                continue
            p = os.path.join(dirpath, fn)
            src = open(p, encoding='utf-8', errors='replace').read()
            if '#[test]' not in src and 'tokio::test' not in src and 'rstest' not in src:
                continue
            rel = os.path.relpath(p, root)
            code = strip_strings_comments(src)
            for line, name, body in find_fns(src, code):
                flags, n_assert = classify(body)
                rows.append((rel, line, name, n_assert, body.count('\n') + 1,
                             ','.join(flags),
                             hashlib.sha1(re.sub(r'\s+', ' ', strip_strings_comments(body)).encode()).hexdigest()[:12]))
    with open(out_path, 'w') as f:
        f.write('file\tline\ttest\tn_assert\tn_lines\tflags\tbodyhash\n')
        for r in sorted(rows, key=lambda r: (r[0], r[1])):
            f.write('\t'.join(str(x) for x in r) + '\n')

    flag_counts = Counter(fl for r in rows for fl in r[5].split(',') if fl)
    byh = defaultdict(int)
    for r in rows:
        byh[r[6]] += 1
    dup_tests = sum(v for v in byh.values() if v > 1)
    print(f'test fns: {len(rows)}  files: {len(set(r[0] for r in rows))}')
    for k, v in flag_counts.most_common():
        print(f'  {v:5d}  {k}')
    print(f'  duplicate-body tests (identical after blanking literals): {dup_tests}')


if __name__ == '__main__':
    main()
