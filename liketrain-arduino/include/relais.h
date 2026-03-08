#ifndef RELAIS_H
#define RELAIS_H

#include <Arduino.h>

class Relais
{
private:
    uint8_t pin;
    uint8_t on_state = HIGH;

public:
    Relais(uint8_t pin) : pin(pin) {}
    Relais(uint8_t pin, uint8_t on_state) : pin(pin), on_state(on_state) {}

    inline void init()
    {
        pinMode(pin, OUTPUT);
        off();
    }

    inline void on() { digitalWrite(pin, on_state); }
    inline void off() { digitalWrite(pin, !on_state); }
};

#endif