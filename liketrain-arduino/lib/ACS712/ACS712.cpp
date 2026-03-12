#include "ACS712.h"

void ACS712::begin()
{
    pinMode(pin, INPUT);
    last_sampled_micros = micros();
}

void ACS712::update()
{
    unsigned long now = micros();

    if (now - last_sampled_micros < SAMPLE_INTERVAL_MICROS)
        return;

    last_sampled_micros = now;

    sample();
}

void ACS712::sample()
{
    int raw = analogRead(pin);
    float centered = raw - offset;

    sum_squares += centered * centered;
    samples++;

    if (samples >= SAMPLE_COUNT)
    {
        compute_rms();
        reset_samples();
    }
}

void ACS712::compute_rms()
{
    float mean = sum_squares / samples;
    float rms_counts = sqrt(mean);

    float volts_per_count = 5.0 / 1023.0;

    rms_voltage = rms_counts * volts_per_count;
    value_ready = true;
}

void ACS712::reset_samples()
{
    samples = 0;
    sum_squares = 0.0;
}