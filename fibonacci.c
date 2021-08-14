#include <stdio.h>

// gcc fibonacci.c -o fibonacci

int fib(int n) {
    asm (".loop:"
         "xadd %1, %2;"
         "loop .loop"
         : "=a"(n)
         : "a"(1), "d"(0), "c"(n)
        );
    
    return n;
}

int main(void) {
    int i;

    for (i = 1; i <= 16; ++i) {
        printf("%d => %d\n", i, fib(i));
    }
}
