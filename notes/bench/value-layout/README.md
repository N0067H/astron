# Value Layout Benchmark

작성일: 2026-08-21

## 대상

- 변경 대상: [src/interpreter.rs](/home/noobth/github/astron/src/interpreter.rs:5)
- 최적화 내용:
- `Value::Str(String)` -> `Value::Str(Rc<String>)`
- `Value::Array(Vec<Value>)` -> `Value::Array(Rc<Vec<Value>>)`
- `Value::Enum { String, String }` -> `Value::Enum { Rc<String>, Rc<String> }`

## 벤치 코드

- 파일: [bench.rs](/home/noobth/github/astron/notes/bench/value-layout/bench.rs)
- 측정 명령:

```sh
rustc -O notes/bench/value-layout/bench.rs -o /tmp/value-layout-bench
/tmp/value-layout-bench
```

## 측정 방식

- `OldValue`와 `NewValue`를 각각 80,000개 생성
- 각 trial마다 clone + pattern-match scan 30회 수행
- 10회 반복 측정

## 구조 크기

- before: `48 bytes`
- after: `24 bytes`

## 측정값

- before(ms): `37.667, 41.839, 52.052, 44.763, 40.770, 54.080, 44.314, 39.337, 37.443, 48.262`
- after(ms): `4.924, 5.223, 4.782, 4.993, 4.270, 5.075, 3.728, 7.070, 4.014, 3.214`
- before 평균: `39.379ms`
- after 평균: `3.873ms`
- 개선율: `90.2%`

## 해석

- 값 enum 자체가 절반 크기로 줄어 한 cache line에 더 많은 값 헤더가 실린다.
- 문자열/배열/enum clone이 deep copy에서 shared pointer clone으로 바뀌어 hot path 메모리 트래픽이 크게 줄었다.
