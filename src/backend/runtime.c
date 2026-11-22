#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <math.h>
#include <inttypes.h>
#include <stdarg.h>
typedef struct V { int k; int64_t i; double f; const char *s; size_t n; const struct V *a; } V;
static V vi(int64_t i) { return (V){0,i,0,0}; }
static V vf(double f) { return (V){1,0,f,0}; }
static V vb(int b) { return (V){2,b,0,0}; }
static V vy(int b) { return (V){3,b,0,0}; }
static V vs(const char *s, size_t n) { return (V){4,0,0,s,n}; }
static V air(void) { return (V){5,0,0,0}; }
static V ve(const char *s, size_t n) { return (V){6,0,0,s,n,0}; }
static void fail(const char *location, const char *format, ...) {
    fprintf(stderr,"%s: runtime error: ",location);
    va_list args;
    va_start(args,format);
    vfprintf(stderr,format,args);
    va_end(args);
    fputc('\n',stderr);
    exit(1);
}
static const char *kind(V v) {
    static const char *names[] = {"int","float","flag","byte","str","void","enum","array"};
    return names[v.k];
}
static void *allocate(size_t n) {
    void *p = malloc(n ? n : 1);
    if (!p) exit(1);
    return p;
}
static V va(const V *items, size_t n) {
    if (n > SIZE_MAX / sizeof(V)) exit(1);
    V *copy = allocate(n * sizeof(V));
    if (n) memcpy(copy,items,n * sizeof(V));
    return (V){7,0,0,0,n,copy};
}
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
    if (a.k == 4 || a.k == 6) return string_compare(a,b) == 0;
    if (a.k == 7) return 0;
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
static V checked_index(V index, const char *location) {
    if (index.k != 0) fail(location,"expected int, got %s",kind(index));
    if (index.i < 0) fail(location,"array index must be non-negative, got %" PRId64,index.i);
    return index;
}
static V array_get(V array, V index, const char *array_location, const char *index_location) {
    index = checked_index(index,index_location);
    if (array.k != 7) fail(array_location,"cannot index into %s",kind(array));
    if ((uint64_t)index.i >= array.n)
        fail(index_location,"array index %" PRId64 " out of bounds with length %zu",index.i,array.n);
    return array.a[index.i];
}
static V array_set(V array, V index, V rhs, int op, const char *location, const char *name) {
    if (array.k != 7) fail(location,"cannot index into %s",kind(array));
    if ((uint64_t)index.i >= array.n)
        fail(location,"array index %" PRId64 " out of bounds for '%s' with length %zu",index.i,name,array.n);
    V result = va(array.a,array.n);
    V *items = (V *)result.a;
    items[index.i] = op < 0 ? rhs : binary(op,items[index.i],rhs);
    return result;
}
static V value_len(V v, const char *location) {
    if (v.k == 7) return vi((int64_t)v.n);
    if (v.k == 4) return string_len(v);
    fail(location,"'len' expects array or str, got %s",kind(v));
    return air();
}
static V assert_value(V v, const char *location) {
    if (v.k != 2) fail(location,"expected flag, got %s",kind(v));
    if (!v.i) fail(location,"assertion failed");
    return air();
}
static V float_string(double f) {
    char buffer[768];
    if (isnan(f)) return vs("NaN",3);
    if (isinf(f)) return f < 0 ? vs("-inf",4) : vs("inf",3);
    if (trunc(f) == f) {
        snprintf(buffer,sizeof(buffer),"%.1f",f);
    } else {
        char shortest[32];
        for (int precision = 1; precision <= 17; precision++) {
            snprintf(shortest,sizeof(shortest),"%.*g",precision,f);
            if (strtod(shortest,NULL) == f) break;
        }
        char *exponent = strchr(shortest,'e');
        if (!exponent) {
            strcpy(buffer,shortest);
        } else {
            int power = atoi(exponent+1);
            char digits[32];
            size_t count = 0;
            int point = 0;
            const char *start = shortest + (f < 0);
            for (const char *p = start; p < exponent; p++) {
                if (*p == '.') point = (int)count;
                else digits[count++] = *p;
            }
            if (!strchr(start,'.')) point = (int)count;
            point += power;
            size_t out = 0;
            if (f < 0) buffer[out++] = '-';
            if (point <= 0) {
                buffer[out++] = '0';
                buffer[out++] = '.';
                for (int i = 0; i < -point; i++) buffer[out++] = '0';
            }
            for (size_t i = 0; i < count; i++) {
                if ((int)i == point) buffer[out++] = '.';
                buffer[out++] = digits[i];
            }
            buffer[out] = 0;
        }
    }
    size_t n = strlen(buffer);
    char *copy = allocate(n);
    memcpy(copy,buffer,n);
    return vs(copy,n);
}
static V to_string(V v) {
    char buffer[32];
    if (v.k == 4 || v.k == 6) return vs(v.s,v.n);
    if (v.k == 5) return vs("",0);
    if (v.k == 2) return v.i ? vs("true",4) : vs("false",5);
    if (v.k == 1) return float_string(v.f);
    if (v.k == 7) {
        V text = vs("[",1);
        for (size_t i = 0; i < v.n; i++) {
            if (i) text = string_add(text,vs(", ",2));
            text = string_add(text,to_string(v.a[i]));
        }
        return string_add(text,vs("]",1));
    }
    if (v.k == 0) snprintf(buffer,sizeof(buffer),"%" PRId64,v.i);
    else snprintf(buffer,sizeof(buffer),"0x%02X",(unsigned)v.i);
    size_t n = strlen(buffer);
    char *copy = allocate(n);
    memcpy(copy,buffer,n);
    return vs(copy,n);
}
static void display(V v) {
    V text = to_string(v);
    fwrite(text.s,1,text.n,stdout);
}
