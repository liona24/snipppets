#include <stdint.h>
#include <stdio.h>

// gcc -O3 -o print_stack_cookie -fstack-protector print_stack_cookie.c

void print_stack_cookie() {
  uint64_t cookie;
  asm("mov %%fs:0x28,%0" : "=r"(cookie));
  printf("cookie: 0x%zx\n", cookie);
}

int main() {
  // some dummy functionality in order to create an incentive for stack
  // protection
  char buf[64];

  print_stack_cookie();

  if (!fgets(buf, sizeof(buf) * 2, stdin)) {
    perror("fgets()");
    return 1;
  }

  return 0;
}
