#include "ACS712Detector.h"

ACS712Detector::ACS712Detector(uint8_t pin) : pin(pin)
{
}

void ACS712Detector::begin()
{
    // Initialize the pin as an input
    pinMode(pin, INPUT);
}

void ACS712Detector::calibrate() {
    // Take multiple samples to determine the offset
    const int sample_count = 50;

    long total = 0;
    for (int i = 0; i < sample_count; i++)
    {
        total += analogRead(pin);
        delay(1); // small delay between samples
    }

    offset = total / sample_count;
}

void ACS712Detector::new_frame() {
    last_frame_start = millis();
    current_frame_peak = 0.0; // Reset the peak for the new frame
}

ACS712DetectorEvent ACS712Detector::update()
{
    int raw = analogRead(pin);

    // offset += (raw - offset) * 0.001; // Slowly adjust offset to account for drift

    int magnitude = abs(raw - offset);

    current_frame_peak = max(current_frame_peak, magnitude);

    if (millis() - last_frame_start < frame_time)
        return ACS712DetectorEvent::None;

    // train has entered
    if (!is_occupied && current_frame_peak > enter_threshold)
    {
        is_occupied = true;
        return ACS712DetectorEvent::Occupied;
    }

    // train has left
    if (is_occupied && current_frame_peak < leave_threshold)
    {
        is_occupied = false;
        return ACS712DetectorEvent::Free;
    }

    new_frame();

    return ACS712DetectorEvent::None;
}

void ACS712Detector::reset()
{
    is_occupied = false;
    offset = 512;
    new_frame();
}