#ifndef ACS712_H
#define ACS712_H

#include <Arduino.h>

class ACS712
{
public:
    ACS712(uint8_t pin) : pin(pin) {}

    void begin();
    void calibrate();

    void update();

    bool available() const { return value_ready; }

    float get_rms()
    {
        value_ready = false;
        return rms_voltage;
    }

    float peek_rms() { return rms_voltage; }
    inline int get_raw() { return analogRead(pin); }

    float get_sum_squares() const { return sum_squares; }
    size_t get_sample_count() const { return samples; }

    unsigned long get_last_sampled_micros() const { return last_sampled_micros; }

private:
    static const unsigned long SAMPLE_INTERVAL_MICROS = 500;
    static const size_t SAMPLE_COUNT = 400;

    uint8_t pin;

    unsigned long last_sampled_micros = 0;

    int offset = 512; // midpoint for 10-bit ADC

    size_t samples = 0;
    float sum_squares = 0.0;

    float rms_voltage = 0.0;
    bool value_ready = false;

    void sample();
    void compute_rms();
    void reset_samples();
};

#endif // ACS712_H