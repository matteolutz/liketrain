#include "ACS712.h"

void ACS712::begin()
{
    pinMode(pin, INPUT);
    last_sampled_micros = micros();
}

void ACS712::calibrate() {
  uint32_t total = 0;
  uint16_t cycles = 10;

  for (uint16_t i = 0; i < cycles; i++)
  {
    uint32_t sub_total = 0;
    uint32_t samples  = 0;
    uint32_t start    = micros();

    while (micros() - start < SAMPLE_INTERVAL_MICROS)
    {
      uint16_t reading = analogRead(pin);
      sub_total += reading;
      samples++;
      //  Delaying prevents overflow
      //  since we'll perform a maximum of 40,000 reads @ 50 Hz.
      delayMicroseconds(1);
    }

    total += (sub_total / samples);
  }
  offset = (total + (cycles/2))/ cycles; 
}

void ACS712::update()
{
    unsigned long now = micros();

    if (now - last_sampled_micros >= SAMPLE_INTERVAL_MICROS)
    {
        last_sampled_micros = now;
        sample();
    }
}

void ACS712::sample()
{
    int raw = analogRead(pin);
    float centered = (float)(raw - offset);

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
    float mean = sum_squares / (float) samples;
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