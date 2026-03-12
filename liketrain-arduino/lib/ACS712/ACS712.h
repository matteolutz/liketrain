#ifndef ACS712_H
#define ACS712_H

#include <Arduino.h>

class ACS712
{
public:
    ACS712(uint8_t pin) : pin(pin) {}

    void begin();

    void update();

    bool available() const { return value_ready; }

    float get_rms()
    {
        value_ready = false;
        return rms_voltage;
    }

private:
    static const uint16_t SAMPLE_INTERVAL_MICROS = 1000;
    static const uint16_t SAMPLE_COUNT = 200;

    uint8_t pin;

    unsigned long last_sampled_micros = 0;

    float offset = 512.0; // midpoint for 10-bit ADC

    uint16_t samples = 0;
    float sum_squares = 0.0;

    float rms_voltage = 0.0;
    bool value_ready = false;

    void sample();
    void compute_rms();
    void reset_samples();
};

#endif // ACS712_H