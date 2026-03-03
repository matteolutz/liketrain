#ifndef SECTION_H
#define SECTION_H

#include <Arduino.h>

enum class SectionPower : uint8_t
{
    Off = 0,
    Quarter = 1,
    Half = 2,
    ThreeQuarters = 3,
    Full = 4
};

class Section
{
};

#endif // SECTION_H