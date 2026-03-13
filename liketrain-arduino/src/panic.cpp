#include "panic.h"
#include "utils.h"

uint8_t __panic_led = NOT_A_PIN;

void panic_init(uint8_t panic_led)
{
    pinMode(panic_led, OUTPUT);
    digitalWrite(panic_led, LOW);

    __panic_led = panic_led;
}

void panic(const char *reason)
{
    UNUSED(reason); // TODO: print reason to serial or something

    if (__panic_led == NOT_A_PIN)
    {
        while(1)
        ;
    }

    while (1)
    {
        digitalWrite(__panic_led, HIGH);
        delay(250);
        digitalWrite(__panic_led, LOW);
        delay(250);
    }
}