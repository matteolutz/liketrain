#ifndef PANIC_H
#define PANIC_H

#include <Arduino.h>

#define DEFAULT_PANIC_LED LED_BUILTIN

/**
 * Initialize the panic system. This should be called at the beginning of setup() to ensure that the panic function can be used safely.
 * 
 * @param panic_led The pin to blink when a panic occurs. If not provided, the default is DEFAULT_PANIC_LED. If set to NOT_A_PIN, the panic function will just loop indefinitely without blinking any LED.
 */
void panic_init(uint8_t panic_led = DEFAULT_PANIC_LED);

/**
 * Panic function that can be called when a critical error occurs. This will blink the panic LED (if initialized) indefinitely to indicate that a panic has occurred. The reason parameter is currently unused, but may be printed to serial or used in some other way in the future to provide more information about the cause of the panic.
 * 
 * @param reason A string describing the reason for the panic. This is currently unused, but may be printed to serial in the future.
 */
void panic(const char *reason = "") __attribute__((noreturn));

#endif // PANIC_H