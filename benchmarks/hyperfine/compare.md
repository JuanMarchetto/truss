| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./target/release/truss validate --quiet benchmarks/fixtures/complex-dynamic.yml` | 2.2 ± 0.8 | 1.3 | 5.9 | 1.00 |
| `competitors/actionlint/run.sh benchmarks/fixtures/complex-dynamic.yml` | 5.9 ± 0.8 | 3.7 | 8.8 | 2.61 ± 0.96 |
| `competitors/yaml-language-server/run.sh benchmarks/fixtures/complex-dynamic.yml` | 336.0 ± 19.8 | 327.2 | 392.1 | 149.48 ± 51.30 |
| `competitors/yamllint/run.sh benchmarks/fixtures/complex-dynamic.yml` | 71.1 ± 2.3 | 65.6 | 77.2 | 31.61 ± 10.74 |
