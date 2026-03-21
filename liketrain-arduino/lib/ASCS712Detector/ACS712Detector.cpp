#include "ACS712Detector.h"

ACS712Detector::ACS712Detector(uint8_t pin) : pin(pin)
{
}

void ACS712Detector::begin()
{
    // Initialize the pin as an input
    pinMode(pin, INPUT);
}

ACS712DetectorEvent ACS712Detector::update()
{
    int raw = analogRead(pin);

    offset += (raw - offset) * 0.001; // Slowly adjust offset to account for drift

    int magnitude = abs(raw - offset);

    filtered += alpha * (magnitude - filtered); // Exponential moving average (low-pass filter)

    // train has entered
    if (!is_occupied && filtered > enter_threshold)
    {
        is_occupied = true;
        return ACS712DetectorEvent::Occupied;
    }

    // train has left
    if (is_occupied && filtered < leave_threshold)
    {
        is_occupied = false;
        return ACS712DetectorEvent::Free;
    }

    return ACS712DetectorEvent::None;
}

void ACS712Detector::reset()
{
    is_occupied = false;
    filtered = 0.0;
    offset = 512;
}