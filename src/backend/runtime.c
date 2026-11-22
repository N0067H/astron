#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <math.h>
#include <inttypes.h>
typedef struct { int k; int64_t i; double f; const char *s; size_t n; } V;
static V vi(int64_t i) { return (V){0,i,0,0}; }
static V vf(double f) { return (V){1,0,f,0}; }
static V vb(int b) { return (V){2,b,0,0}; }
static V vy(int b) { return (V){3,b,0,0}; }
static V vs(const char *s, size_t n) { return (V){4,0,0,s,n}; }
static V air(void) { return (V){5,0,0,0}; }
static int truth(V v) { if (v.k != 2) exit(1); return v.i != 0; }
static int64_t integer(V v) { if (v.k != 0) exit(1); return v.i; }
static double number(V v) { return v.k == 0 ? (double)v.i : v.f; }
static int string_compare(V a, V b) {
    size_t n = a.n < b.n ? a.n : b.n;
    int order = memcmp(a.s,b.s,n);
    return order ? order : (a.n > b.n) - (a.n < b.n);
}
static V string_len(V v) {
    if (v.k != 4) exit(1);
    int64_t count = 0;
    for (size_t i = 0; i < v.n; i++)
        if (((unsigned char)v.s[i] & 0xC0) != 0x80) count++;
    return vi(count);
}
static V string_add(V a, V b) {
    if (a.n > SIZE_MAX - b.n) exit(1);
    size_t n = a.n + b.n;
    char *s = malloc(n ? n : 1);
    if (!s) exit(1);
    memcpy(s,a.s,a.n);
    memcpy(s+a.n,b.s,b.n);
    return vs(s,n);
}
static int equal(V a, V b) {
    if (a.k < 2 && b.k < 2) return a.k == 0 && b.k == 0 ? a.i == b.i : number(a) == number(b);
    if (a.k != b.k) return 0;
    if (a.k == 4) return string_compare(a,b) == 0;
    return a.i == b.i;
}
static V unary(int op, V a) {
    if (op) return vb(!truth(a));
    if (a.k == 0) return vi((int64_t)(0-(uint64_t)a.i));
    if (a.k == 1) return vf(-a.f);
    exit(1);
}
static V binary(int op, V a, V b) {
    if (op == 5) return vb(equal(a,b));
    if (op == 6) return vb(!equal(a,b));
    if (a.k == 4 && b.k == 4) {
        if (op == 0) return string_add(a,b);
        int order = string_compare(a,b);
        if (op == 7) return vb(order < 0);
        if (op == 8) return vb(order > 0);
        if (op == 9) return vb(order <= 0);
        if (op == 10) return vb(order >= 0);
        exit(1);
    }
    if (a.k > 1 || b.k > 1 || (op == 4 && a.k != b.k)) exit(1);
    if (op >= 7) {
        if (a.k == 0 && b.k == 0) {
            if (op == 7) return vb(a.i < b.i);
            if (op == 8) return vb(a.i > b.i);
            if (op == 9) return vb(a.i <= b.i);
            return vb(a.i >= b.i);
        }
        double x = number(a), y = number(b);
        if (op == 7) return vb(x < y);
        if (op == 8) return vb(x > y);
        if (op == 9) return vb(!(x > y));
        return vb(!(x < y));
    }
    if (a.k == 0 && b.k == 0) {
        if (op == 0) return vi((uint64_t)a.i + (uint64_t)b.i);
        if (op == 1) return vi((uint64_t)a.i - (uint64_t)b.i);
        if (op == 2) return vi((uint64_t)a.i * (uint64_t)b.i);
        if (!b.i || (a.i == INT64_MIN && b.i == -1)) exit(1);
        if (op == 3) return vi(a.i / b.i);
        return vi(a.i % b.i);
    }
    double x = number(a), y = number(b);
    if ((op == 3 || op == 4) && y == 0) exit(1);
    if (op == 0) return vf(x+y);
    if (op == 1) return vf(x-y);
    if (op == 2) return vf(x*y);
    if (op == 3) return vf(x/y);
    return vf(fmod(x,y));
}
static void display(V v) {
    if (v.k == 0) printf("%" PRId64, v.i);
    else if (v.k == 1) { if (isfinite(v.f) && trunc(v.f) == v.f) printf("%.1f",v.f); else printf("%.15g",v.f); }
    else if (v.k == 2) fputs(v.i ? "true" : "false",stdout);
    else if (v.k == 3) printf("0x%02X",(unsigned)v.i);
    else if (v.k == 4) fwrite(v.s,1,v.n,stdout);
    else fputs("",stdout);
}
