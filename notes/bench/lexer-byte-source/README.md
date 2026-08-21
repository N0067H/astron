# Lexer Byte Source Benchmark

작성일: 2026-08-21

## 대상

- 변경 대상: [src/lexer.rs](/home/noobth/github/astron/src/lexer.rs:8)
- 최적화 내용:
- `Vec<char>` 기반 소스 버퍼를 `Vec<u8>` 기반으로 전환
- ASCII 위주의 토큰 스캔을 byte 단위로 진행

## 벤치 코드

- 파일: [bench.rs](/home/noobth/github/astron/notes/bench/lexer-byte-source/bench.rs)
- 측정 명령:

```sh
rustc -O notes/bench/lexer-byte-source/bench.rs -o /tmp/lexer-byte-source-bench
/tmp/lexer-byte-source-bench
```

## 측정 방식

- comment, ident, number, punctuation이 섞인 ASCII 소스 20,000 블록 생성
- before는 `Vec<char>` 스캔
- after는 `Vec<u8>` 스캔
- 10회 반복 측정

## 측정값

- before(ms): `7.162, 4.445, 4.085, 6.251, 4.499, 2.987, 2.885, 3.104, 3.251, 4.469`
- after(ms): `2.892, 3.002, 2.081, 3.126, 2.151, 1.802, 1.768, 1.900, 2.080, 2.530`
- before 평균: `3.734ms`
- after 평균: `2.195ms`
- 개선율: `41.2%`

## 해석

- ASCII 중심 입력에서는 `char` 4바이트 저장보다 `u8` 저장이 cache line당 훨씬 많은 입력을 실어 나른다.
- 순차 스캔 루프에서 메모리 대역폭과 branch 주변 locality가 함께 좋아진다.
