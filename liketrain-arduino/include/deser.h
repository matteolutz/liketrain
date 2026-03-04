#ifndef DESER_H
#define DESER_H

#include <Arduino.h>

#include "queue.h"
#include "utils.h"

#include "RS485.h"

#define DESER_SERIAL_BAUD 115200

#define DESER_SERIAL_BUFFER_SIZE 128
#define DESER_SERIAL_QUEUE_SIZE 256

#define DESER_SERIAL_START_BYTE 0xAA

class DeserSerializer
{
public:
    virtual void write_bytes(const uint8_t *data, size_t size) = 0;

    void write_u8(uint8_t value)
    {
        write(value);
    }

    void write_u16(uint16_t value)
    {
        write_u8(value & 0xFF);        // low byte
        write_u8((value >> 8) & 0xFF); // high byte
    }

    void write_u32(uint32_t value)
    {
        write_u16(value & 0xFFFF);         // low word
        write_u16((value >> 16) & 0xFFFF); // high word
    }

    template <typename T>
    void write(const T &value)
    {
        write_bytes(
            reinterpret_cast<const uint8_t *>(&value),
            sizeof(T));
    }
};

class DeserDeserializer
{
public:
    virtual void read_bytes(uint8_t *data, size_t size) = 0;

    uint8_t read_u8()
    {
        uint8_t value;
        read(value);
        return value;
    }

    uint16_t read_u16()
    {
        uint8_t low = read_u8();
        uint8_t high = read_u8();
        return (static_cast<uint16_t>(high) << 8) | low;
    }

    uint32_t read_u32()
    {
        uint16_t low = read_u16();
        uint16_t high = read_u16();
        return (static_cast<uint32_t>(high) << 16) | low;
    }

    template <typename T>
    void read(T &value)
    {
        read_bytes(
            reinterpret_cast<uint8_t *>(&value),
            sizeof(T));
    }
};

class DeserHeapBufferDeserializer : public DeserDeserializer
{
    uint8_t *buffer = nullptr;
    size_t buffer_size = 0;
    size_t position = 0;

public:
    DeserHeapBufferDeserializer() {}
    ~DeserHeapBufferDeserializer()
    {
        if (buffer != nullptr)
            delete[] buffer;
    }

    void load(const uint8_t *buf, size_t size)
    {
        if (buffer != nullptr)
            delete[] buffer;

        buffer = new uint8_t[size];
        memcpy(buffer, buf, size);

        buffer_size = size;
        position = 0;
    }

    void read_bytes(uint8_t *data, size_t size) override
    {
        memcpy(data, buffer + position, size);
        position += size;
    }
};

template <size_t N>
class DeserBufferSerializer : public DeserSerializer
{
    uint8_t buffer[N];
    size_t position = 0;

public:
    uint8_t *data() { return buffer; }
    size_t size() const { return position; }

    void reset()
    {
        position = 0;
    }

public:
    void write_bytes(const uint8_t *data, size_t size) override
    {
        if (position + size > N)
            return; // or handle error

        memcpy(buffer + position, data, size);
        position += size;
    }
};

class DeserInterface
{
public:
    virtual void serialize(DeserSerializer &ser) const = 0;
    virtual void deserialize(DeserDeserializer &deser) = 0;
};

enum class DeserSerialSerialImplementation
{
    HardwareSerial,
    RS485
};

class DeserSerial
{
private:
    // TODO: maybe jsut use a Stream* instead of having separate pointers and an enum for the implementation type?
    HardwareSerial *hw_serial = nullptr;
    RS485 *rs485 = nullptr;

    DeserSerialSerialImplementation impl;

    uint8_t rx_buffer[DESER_SERIAL_BUFFER_SIZE] = {};

    uint8_t rx_queue_buffer[DESER_SERIAL_QUEUE_SIZE] = {};
    Queue<uint8_t> rx_queue;

    bool read_command(uint8_t *payload, size_t payload_length);

    static uint8_t checksum(uint8_t *payload, size_t payload_length);

    int serial_available();

    int serial_read_byte();
    size_t serial_read_bytes(uint8_t *buffer, size_t max_size);

    size_t serial_write_bytes(const uint8_t *buffer, size_t size);
    size_t serial_write_byte(uint8_t byte);

    void serial_flush();

public:
    DeserSerial(HardwareSerial &serial);
    DeserSerial(RS485 &rs485);

    void init();
    void update();

    bool read_frame(DeserHeapBufferDeserializer &deserializer);
    bool await_frame(DeserHeapBufferDeserializer &deserializer, unsigned long timeout_ms);

    template <size_t N>
    void write_frame(DeserBufferSerializer<N> &serializer)
    {
        serial_write_byte(DESER_SERIAL_START_BYTE);

        uint8_t size_bytes[4];
        u32_to_le_bytes(serializer.size(), size_bytes);

        serial_write_bytes(size_bytes, 4);

        serial_write_bytes(serializer.data(), serializer.size());

        uint8_t cs = checksum(serializer.data(), serializer.size());
        serial_write_byte(cs);

        serial_flush();
    }
};

#endif // DESER_H