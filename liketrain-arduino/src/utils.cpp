#include "utils.h"

uint32_t u32_from_le_bytes(uint8_t bytes[4])
{
    return (static_cast<uint32_t>(bytes[0]) << 0) |
           (static_cast<uint32_t>(bytes[1]) << 8) |
           (static_cast<uint32_t>(bytes[2]) << 16) |
           (static_cast<uint32_t>(bytes[3]) << 24);
}

void u32_to_le_bytes(uint32_t value, uint8_t bytes[4])
{
    bytes[0] = (value >> 0) & 0xFF;  // low byte
    bytes[1] = (value >> 8) & 0xFF;  // second byte
    bytes[2] = (value >> 16) & 0xFF; // third byte
    bytes[3] = (value >> 24) & 0xFF; // high byte
}