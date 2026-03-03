#ifndef SWITCH_H
#define SWITCH_H

#include <Arduino.h>

#define SWITCH_ID_LEN 32
using SwitchId = uint8_t[SWITCH_ID_LEN];

enum class SwitchState : uint8_t
{
    Left = 0,
    Right = 1
};

#endif // SWITCH_H