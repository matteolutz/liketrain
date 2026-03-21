#ifndef ACS712_DETECTOR_H
#define ACS712_DETECTOR_H

#include <Arduino.h>

enum class ACS712DetectorEvent
{
    None = -1,
    Occupied,
    Free
};

class ACS712Detector
{
public:
    ACS712Detector(uint8_t pin);

    void begin();
    ACS712DetectorEvent update();

    float get_filtered_value() const { return filtered; }

    bool occupied() const { return is_occupied; }

    void reset();

private:
    uint8_t pin;

    ACS712DetectorEvent last_event = ACS712DetectorEvent::None;
    bool is_occupied = false;

    float filtered = 0.0;
    float alpha = 0.05;

    int offset = 512; // Midpoint for 10-bit ADC

    const float enter_threshold = 30.0;
    const float leave_threshold = 20.0;
};

#endif // ACS712_DETECTOR_H