#include "deser.h"
#include "utils.h"

DeserSerial::DeserSerial(HardwareSerial &serial)
    : hw_serial(&serial),
      impl(DeserSerialSerialImplementation::HardwareSerial),
      rx_queue(rx_queue_buffer, DESER_SERIAL_QUEUE_SIZE)
{
}

DeserSerial::DeserSerial(RS485 &rs485)
    : rs485(&rs485),
      impl(DeserSerialSerialImplementation::RS485),
      rx_queue(rx_queue_buffer, DESER_SERIAL_QUEUE_SIZE)
{
}

void DeserSerial::init()
{
    switch (impl)
    {
    case DeserSerialSerialImplementation::HardwareSerial:
        hw_serial->begin(DESER_SERIAL_BAUD);
        break;
    case DeserSerialSerialImplementation::RS485:
        rs485->init(DESER_SERIAL_BAUD);
        break;
    }

    // clear the input_buffer
    while (serial_available() > 0)
    {
        serial_read_byte();
    }
}

int DeserSerial::serial_available()
{
    switch (impl)
    {
    case DeserSerialSerialImplementation::HardwareSerial:
        return hw_serial->available();
    case DeserSerialSerialImplementation::RS485:
        return rs485->available();
    }

    return 0;
}

int DeserSerial::serial_read_byte()
{
    switch (impl)
    {
    case DeserSerialSerialImplementation::HardwareSerial:
        return hw_serial->read();
    case DeserSerialSerialImplementation::RS485:
        return rs485->read();
    }

    return 0;
}

size_t DeserSerial::serial_read_bytes(uint8_t *buffer, size_t max_size)
{
    switch (impl)
    {
    case DeserSerialSerialImplementation::HardwareSerial:
        return hw_serial->readBytes(buffer, max_size);
    case DeserSerialSerialImplementation::RS485:
        return rs485->readBytes(buffer, max_size);
    }

    return 0;
}

size_t DeserSerial::serial_write_bytes(const uint8_t *buffer, size_t size)
{
    switch (impl)
    {
    case DeserSerialSerialImplementation::HardwareSerial:
        return hw_serial->write(buffer, size);
    case DeserSerialSerialImplementation::RS485:
        return rs485->write(buffer, size);
    }

    return 0;
}

size_t DeserSerial::serial_write_byte(uint8_t byte)
{
    switch (impl)
    {
    case DeserSerialSerialImplementation::HardwareSerial:
        return hw_serial->write(byte);
    case DeserSerialSerialImplementation::RS485:
        return rs485->write(byte);
    }

    return 0;
}

void DeserSerial::serial_flush()
{
    switch (impl)
    {
    case DeserSerialSerialImplementation::HardwareSerial:
        hw_serial->flush();
        break;
    case DeserSerialSerialImplementation::RS485:
        rs485->flush();
        break;
    }
}

void DeserSerial::update()
{
    int available = serial_available();
    if (available <= 0)
    {
        return;
    }
   
    // read all available bytes (max DESER_SERIAL_BUFFER_SIZE at a time) into the rx_buffer
    size_t bytes_read = serial_read_bytes(rx_buffer, min(DESER_SERIAL_BUFFER_SIZE, available));

    if (bytes_read <= 0)
    {
        return;
    }

    // add the bytes from the rx_buffer into the rx_queue
    rx_queue.extend_from(rx_buffer, bytes_read);
}

bool DeserSerial::read_frame(DeserHeapBufferDeserializer &deserializer)
{
    while (1)
    {
        if (rx_queue.is_empty())
            return false; // stop if no data

        uint8_t start_byte = 0;
        rx_queue.peek(start_byte); // will not fail because we checked is_empty() above

        if (start_byte != DESER_SERIAL_START_BYTE)
        {
            // invalid start byte, discard and try again
            rx_queue.drain_front(1);
            continue; // continue reading frames
        }

        // 1 start byte, 4 bytes for size, 1 byte for checksum
        if (rx_queue.size() < 6)
        {
            // not enough data for a complete frame, wait for more
            return false; // stop reading frames
        }

        // read payload size
        uint8_t size_bytes[4];
        for (int i = 0; i < 4; i++)
        {
            // don't dequeue yet, we need to peek at the size bytes to know if we have the full frame in the queue
            rx_queue.peek_at(i + 1, size_bytes[i]);
        }
        uint32_t payload_size = u32_from_le_bytes(size_bytes);

        if (rx_queue.size() < (1 + 4 + payload_size + 1))
        {
            // not enough data for the complete payload and checksum, wait for more
            return false; // stop reading frames
        }

        // we have a complete frame, now dequeue the start and size bytes
        rx_queue.drain_front(1 + 4);

        uint8_t payload[payload_size];
        rx_queue.drain_into(payload, payload_size); // will not fail because we checked size above

        uint8_t received_checksum;
        rx_queue.dequeue(received_checksum); // will not fail because we checked size above

        uint8_t calculated_checksum = checksum(payload, payload_size);

        if (received_checksum != calculated_checksum)
        {
            // checksum mismatch, discard frame and try again
            continue; // continue reading frames
        }

        deserializer.load(payload, payload_size);

        return true;
    }
}

bool DeserSerial::await_frame(DeserHeapBufferDeserializer &deserializer, unsigned long timeout_ms)
{
    unsigned long start_time = millis();

    while (millis() - start_time < timeout_ms)
    {
        if (read_frame(deserializer))
        {
            return true;
        }

        update();
    }

    return false; // timeout
}

uint8_t DeserSerial::checksum(uint8_t *payload, size_t payload_length)
{
    uint8_t sum = 0;
    for (size_t i = 0; i < payload_length; i++)
    {
        sum += payload[i];
    }
    return sum;
}