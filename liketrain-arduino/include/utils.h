#ifndef UTILS_H
#define UTILS_H

#include <Arduino.h>

#define UNUSED(x) (void)(x)

uint32_t u32_from_le_bytes(uint8_t bytes[4]);
void u32_to_le_bytes(uint32_t value, uint8_t bytes[4]);

#endif // UTILS_H