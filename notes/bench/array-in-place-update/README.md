# Array In-Place Update Benchmark

작성일: 2026-08-21

## 대상

- 변경 대상: [src/interpreter.rs](/home/noobth/github/astron/src/interpreter.rs:252)
- 최적화 내용:
- 배열 대입 시 전체 배열을 clone해서 `env.set()`으로 되돌리는 경로 제거
- `Env::get_mut()` + `Rc::make_mut()`으로 제자리 갱신

## 벤치 코드

- 파일: [bench.rs](/home/noobth/github/astron/notes/bench/array-in-place-update/bench.rs)
- 측정 명령:

```sh
rustc -O notes/bench/array-in-place-update/bench.rs -o /tmp/array-in-place-update-bench
/tmp/array-in-place-update-bench
```

## 측정 방식

- 길이 1024 배열에 대해 index update 200,000회 반복
- before는 매번 배열 전체 clone 후 재대입
- after는 `Rc::make_mut()`로 동일 배열 버퍼를 직접 수정

## 측정값

- before(ms): `15.655, 17.946, 19.250, 15.551, 15.741, 15.598, 16.414, 18.822, 16.242, 13.289`
- after(ms): `0.486, 0.293, 0.348, 0.350, 0.556, 0.610, 0.376, 0.618, 0.303, 0.318`
- before 평균: `13.767ms`
- after 평균: `0.304ms`
- 개선율: `97.8%`

## 해석

- 이전 방식은 update 한 번마다 배열 전체를 새로 복사해서 cache line과 allocator를 같이 낭비했다.
- 변경 후에는 같은 버퍼를 유지한 채 필요한 slot만 만져서 메모리 이동량이 거의 사라진다.
