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
    void calibrate();

    ACS712DetectorEvent update();

    int get_frame_peak() const { return current_frame_peak; }

    bool occupied() const { return is_occupied; }

    void reset();

    uint8_t get_pin() const { return pin; }

private:
    void new_frame();

    uint8_t pin;

    ACS712DetectorEvent last_event = ACS712DetectorEvent::None;
    bool is_occupied = false;

    int offset = 512; // Midpoint for 10-bit ADC

    int current_frame_peak = 0.0;

    unsigned long last_frame_start = 0;
    const unsigned long frame_time = 20; // ms
    
    const int enter_threshold = 30;
    const int leave_threshold = 20;

    unsigned long last_event_time = 0;
    const unsigned long event_debounce_time = 1000;
};

#endif // ACS712_DETECTOR_H