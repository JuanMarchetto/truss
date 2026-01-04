| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./target/release/truss validate --quiet benchmarks/fixtures/complex-dynamic.yml` | 2.2 ± 0.7 | 1.1 | 6.6 | 1.00 |
| `competitors/actionlint/run.sh benchmarks/fixtures/complex-dynamic.yml` | 5.9 ± 0.9 | 3.7 | 9.3 | 2.67 ± 0.97 |
| `competitors/yaml-language-server/run.sh benchmarks/fixtures/complex-dynamic.yml` | 340.9 ± 11.3 | 330.8 | 369.8 | 153.72 ± 50.71 |
| `competitors/yamllint/run.sh benchmarks/fixtures/complex-dynamic.yml` | 71.5 ± 2.4 | 67.1 | 77.6 | 32.22 ± 10.63 |
