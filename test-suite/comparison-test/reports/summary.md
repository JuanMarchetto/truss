# Validation Tool Comparison Report

**Generated:** 2026-01-24T20:24:31.311473Z

## Summary

- **Total Files Analyzed:** 4
- **Total Errors (Truss):** 5
- **Total Errors (actionlint):** 0
- **Truss Coverage:** 100.0%
- **Avg Time (Truss):** 2.59ms
- **Avg Time (actionlint):** 0.00ms
- **Speedup:** 0.0x

## Coverage Analysis

### actionlint

- **Total Errors:** 0
- **Truss Found:** 0
- **Coverage:** 100.0%

### yamllint

- **Total Errors:** 0
- **Truss Found:** 0
- **Coverage:** 100.0%

### yaml-language-server

- **Total Errors:** 0
- **Truss Found:** 0
- **Coverage:** 100.0%

## Tool Statistics

| Tool | Files Analyzed | Errors Found | Avg Time (ms) | Total Time (ms) |
|------|----------------|--------------|---------------|-----------------|
| truss | 4 | 5 | 2.59 | 10.35 |
| actionlint | 0 | 0 | 0.00 | 0.00 |
| yamllint | 0 | 0 | 0.00 | 0.00 |
| yaml-language-server | 0 | 0 | 0.00 | 0.00 |

## File-by-File Breakdown

### /home/marche/truss/scripts/../benchmarks/fixtures/complex-dynamic.yml

**Tool Results:**
- **truss:** 2 errors, 3.12ms
- **actionlint:** 0 errors, 0.00ms
- **yamllint:** 0 errors, 0.00ms
- **yaml-language-server:** 0 errors, 0.00ms

**Comparison:**
- vs **actionlint:** 0 common, 2 unique to Truss, 0 unique to actionlint
- vs **yamllint:** 0 common, 2 unique to Truss, 0 unique to yamllint
- vs **yaml-language-server:** 0 common, 2 unique to Truss, 0 unique to yaml-language-server

### /home/marche/truss/scripts/../benchmarks/fixtures/complex-static.yml

**Tool Results:**
- **truss:** 2 errors, 3.82ms
- **actionlint:** 0 errors, 0.00ms
- **yamllint:** 0 errors, 0.00ms
- **yaml-language-server:** 0 errors, 0.00ms

**Comparison:**
- vs **actionlint:** 0 common, 2 unique to Truss, 0 unique to actionlint
- vs **yamllint:** 0 common, 2 unique to Truss, 0 unique to yamllint
- vs **yaml-language-server:** 0 common, 2 unique to Truss, 0 unique to yaml-language-server

### /home/marche/truss/scripts/../benchmarks/fixtures/medium.yml

**Tool Results:**
- **truss:** 1 errors, 2.63ms
- **actionlint:** 0 errors, 0.00ms
- **yamllint:** 0 errors, 0.00ms
- **yaml-language-server:** 0 errors, 0.00ms

**Comparison:**
- vs **actionlint:** 0 common, 1 unique to Truss, 0 unique to actionlint
- vs **yamllint:** 0 common, 1 unique to Truss, 0 unique to yamllint
- vs **yaml-language-server:** 0 common, 1 unique to Truss, 0 unique to yaml-language-server

### /home/marche/truss/scripts/../benchmarks/fixtures/simple.yml

**Tool Results:**
- **truss:** 0 errors, 0.80ms
- **actionlint:** 0 errors, 0.00ms
- **yamllint:** 0 errors, 0.00ms
- **yaml-language-server:** 0 errors, 0.00ms

**Comparison:**
- vs **actionlint:** 0 common, 0 unique to Truss, 0 unique to actionlint
- vs **yamllint:** 0 common, 0 unique to Truss, 0 unique to yamllint
- vs **yaml-language-server:** 0 common, 0 unique to Truss, 0 unique to yaml-language-server

