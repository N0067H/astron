# Env Symbol Table Benchmark

작성일: 2026-08-21

## 대상

- 변경 대상: [src/interpreter.rs](/home/noobth/github/astron/src/interpreter.rs:84)
- 최적화 내용:
- `scopes: Vec<Vec<String>>` -> `scopes: Vec<Vec<usize>>`
- `values: HashMap<String, Vec<Value>>` -> `symbols: HashMap<String, usize>` + `values: Vec<Vec<Value>>`
- 스코프에 변수명을 복제해서 넣던 경로를 symbol id 기록으로 변경

## 벤치 코드

- 파일: [bench.rs](/home/noobth/github/astron/notes/bench/env-symbol-table/bench.rs)
- 측정 명령:

```sh
rustc -O notes/bench/env-symbol-table/bench.rs -o /tmp/env-symbol-table-bench
/tmp/env-symbol-table-bench
```

## 측정 방식

- 긴 변수명 256개 사용
- 각 trial마다 1,200회 scope push/define/lookup/set/pop 반복
- lookup만이 아니라 define/pop churn을 같이 포함해 symbol-id 전환 효과를 보이도록 구성

## 측정값

- before(ms): `45.873, 43.898, 48.076, 46.093, 47.016, 37.245, 42.535, 43.143, 43.084, 37.479`
- after(ms): `26.176, 23.530, 24.420, 24.357, 28.556, 20.341, 20.199, 24.996, 24.063, 26.482`
- before 평균: `43.503ms`
- after 평균: `25.221ms`
- 개선율: `42.0%`

## 해석

- 스코프 스택이 문자열 배열 대신 작은 정수 배열을 저장해서 메모리 밀도가 좋아졌다.
- `define()`와 `pop_scope()`에서 문자열 clone/hash 재접근이 줄어 scope churn workload에서 이득이 크게 났다.
