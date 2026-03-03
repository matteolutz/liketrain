#ifndef PANIC_H
#define PANIC_H

#include <Arduino.h>

#define DEFAULT_PANIC_LED LED_BUILTIN

void panic_init(uint8_t panic_led = DEFAULT_PANIC_LED);
void panic(const char *reason = "") __attribute__((noreturn));

#endif // PANIC_H