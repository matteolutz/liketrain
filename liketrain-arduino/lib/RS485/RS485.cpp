#include "RS485.h"

RS485::RS485(Stream &stream, uint8_t re_de_pin)
    : stream(&stream), re_de_pin(re_de_pin)
{
}

RS485::RS485(HardwareSerial &hw_serial, uint8_t re_de_pin, uint8_t re_de_pin2)
    : stream(&hw_serial), hw_serial(&hw_serial), stream_impl(RS485StreamImplementation::HardwareSerial), re_de_pin(re_de_pin), re_de_pin2(re_de_pin2)
{
}

void RS485::init(unsigned long baud) const
{
    pinMode(re_de_pin, OUTPUT);

    if (re_de_pin2 != NOT_A_PIN)
        pinMode(re_de_pin2, OUTPUT);

    switch (stream_impl)
    {
    case RS485StreamImplementation::HardwareSerial:
        hw_serial->begin(baud);
        break;
    case RS485StreamImplementation::Unknown:
        break;
    }
}

void RS485::flush()
{
    stream->flush();

    // wait for the last byte to be sent before switching back to receive mode.
    // this is copied from https://github.com/RobTillaart/RS485
    delayMicroseconds(RS485_MICROS_PER_BYTE);
}

size_t RS485::write(uint8_t byte)
{
    set_transmit_mode();

    size_t bytes_written = stream->write(byte);
    stream->flush();

    set_receive_mode();

    return bytes_written;
}

size_t RS485::write(const uint8_t *buffer, size_t size)
{
    set_transmit_mode();

    size_t bytes_written = stream->write(buffer, size);
    stream->flush();

    set_receive_mode();

    return bytes_written;
}