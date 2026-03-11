#ifndef SECTION_POWER_H
#define SECTION_POWER_H

#include <Arduino.h>

enum class SectionPower : uint8_t
{
    Off = 0,
    Quarter = 1,
    Half = 2,
    ThreeQuarters = 3,
    Full = 4
};


#endif // SECTION_POWER_H