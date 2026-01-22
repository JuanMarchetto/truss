| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./target/release/truss validate --quiet benchmarks/fixtures/complex-dynamic.yml` | 11.1 ± 5.0 | 1.7 | 17.9 | 1.00 |
| `competitors/actionlint/run.sh benchmarks/fixtures/complex-dynamic.yml` | 165.7 ± 13.4 | 144.5 | 189.7 | 14.98 ± 6.90 |
| `competitors/yaml-language-server/run.sh benchmarks/fixtures/complex-dynamic.yml` | 381.7 ± 104.9 | 263.9 | 553.4 | 34.51 ± 18.30 |
| `competitors/yamllint/run.sh benchmarks/fixtures/complex-dynamic.yml` | 210.9 ± 53.4 | 94.6 | 276.7 | 19.07 ± 9.90 |
