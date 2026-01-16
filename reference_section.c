#include <stdint.h>
#include <stdio.h>

__attribute((__section__("my_section"))) uint8_t x[] = {0x41, 0x41, 0x41, 0x0a};
extern uint8_t __start_my_section;
extern uint8_t __stop_my_section;

int main() {
    for (const uint8_t* it = &__start_my_section; it != &__stop_my_section; it++) {
        putchar(*it);
    }

    return 0;
}
