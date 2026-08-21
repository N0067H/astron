# Interpreter Metadata Benchmark

작성일: 2026-08-21

## 대상

- 변경 대상: [src/interpreter.rs](/home/noobth/github/astron/src/interpreter.rs:161)
- 최적화 내용:
- `Interpreter`가 stage params/body를 `HashMap<String, (Vec<Param>, Vec<Stmt>)>`로 복제하지 않도록 변경
- 프로그램 AST를 직접 참조하고 함수 테이블은 `HashMap<&str, usize>` 인덱스만 유지

## 벤치 코드

- 파일: [bench.rs](/home/noobth/github/astron/notes/bench/interpreter-metadata/bench.rs)
- 측정 명령:

```sh
rustc -O notes/bench/interpreter-metadata/bench.rs -o /tmp/interpreter-metadata-bench
/tmp/interpreter-metadata-bench
```

## 측정 방식

- stage 1,500개, 각 stage당 param 8개와 stmt 96개로 synthetic program 구성
- 각 trial마다 interpreter metadata 구성 + dispatch checksum 60회 반복
- before는 params/body clone 포함
- after는 프로그램 참조 + 인덱스 맵만 생성

## 측정값

- before(ms): `58.415, 65.326, 58.879, 54.567, 54.363, 68.449, 58.798, 52.609, 58.961, 53.079`
- after(ms): `9.094, 7.366, 8.732, 7.851, 8.072, 8.666, 8.716, 7.578, 8.577, 10.966`
- before 평균: `57.481ms`
- after 평균: `8.698ms`
- 개선율: `84.9%`

## 해석

- 런타임 시작 시 큰 AST 조각을 다시 복제하지 않아 working set이 크게 줄었다.
- 함수 테이블이 이름과 인덱스만 들고 있어서 metadata 순회 밀도도 좋아졌다.
