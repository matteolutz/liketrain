#ifndef RS485_H
#define RS485_H

#include <Arduino.h>

#define RS485_MICROS_PER_BYTE 1100

enum class RS485Mode
{
    Receive,
    Transmit
};

enum class RS485StreamImplementation
{
    Unknown,
    HardwareSerial,
};

class RS485 : public Stream
{
public:
    RS485(Stream &stream, uint8_t re_de_pin);

    RS485(HardwareSerial &hw_serial, uint8_t re_de_pin, uint8_t re_de_pin2 = NOT_A_PIN);

    // ~~~~~~~~~~~ Stream interface ~~~~~~~~~~ //
    inline int available() override { return stream->available(); }
    inline int read() override { return stream->read(); }
    inline int peek() override { return stream->peek(); }

    // ~~~~~~~~~~~ Print interface ~~~~~~~~~~~ //
    void flush() override;

    size_t write(uint8_t byte) override;
    size_t write(const uint8_t *buffer, size_t size) override;

    // ~~~~~~~~ RS485 specific methods ~~~~~~~ //
    void init(unsigned long baud = 115200) const;

    inline void set_mode(RS485Mode mode)
    {
        uint8_t level = mode == RS485Mode::Transmit ? HIGH : LOW;

        digitalWrite(re_de_pin, level);
        if (re_de_pin2 != NOT_A_PIN)
            digitalWrite(re_de_pin2, level);
    };

    inline RS485Mode get_mode() const { return digitalRead(re_de_pin) == LOW ? RS485Mode::Receive : RS485Mode::Transmit; }

    inline void set_receive_mode() { set_mode(RS485Mode::Receive); }
    inline void set_transmit_mode() { set_mode(RS485Mode::Transmit); }

private:
    Stream *stream;

    HardwareSerial *hw_serial = nullptr;
    RS485StreamImplementation stream_impl = RS485StreamImplementation::Unknown;

    uint8_t re_de_pin;
    uint8_t re_de_pin2 = NOT_A_PIN;
};

#endif // RS485_H