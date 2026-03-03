#ifndef RESPONSE_H
#define RESPONSE_H

#include "deser.h"
#include "event.h"

enum class LiketrainResponseType : uint8_t
{
    Invalid = 0x0,

    Ack = 0x1,
    DebugMessage = 0x2,

    Event = 0x10
};

union LiketrainResponseData
{
    struct
    {
        uint32_t len;
        char *message;
    } debug_message;

    struct
    {
        LiketrainEvent *event;
    } event;
};

class LiketrainResponse : public DeserInterface
{
public:
    LiketrainResponseType type = LiketrainResponseType::Invalid;
    LiketrainResponseData data = {0};

    virtual ~LiketrainResponse()
    {
        switch (type)
        {
        case LiketrainResponseType::DebugMessage:
            delete[] data.debug_message.message;
            break;
        case LiketrainResponseType::Event:
            delete data.event.event;
            break;
        }
    }

public:
    static LiketrainResponse ack()
    {
        LiketrainResponse response;
        response.type = LiketrainResponseType::Ack;
        return response;
    }

    static LiketrainResponse event(LiketrainEvent &event)
    {
        LiketrainResponse response;
        response.type = LiketrainResponseType::Event;
        response.data.event.event = new LiketrainEvent(event);
        return response;
    }

public:
    void serialize(DeserSerializer &ser) const override
    {
        ser.write_u8(static_cast<uint8_t>(type));

        switch (type)
        {
        case LiketrainResponseType::Invalid:
            break;
        case LiketrainResponseType::DebugMessage:
            ser.write_u32(data.debug_message.len);
            ser.write_bytes(reinterpret_cast<const uint8_t *>(data.debug_message.message), data.debug_message.len);
            break;
        case LiketrainResponseType::Event:
            data.event.event->serialize(ser);
            break;
        }
    }

    void deserialize(DeserDeserializer &deser) override
    {
        type = static_cast<LiketrainResponseType>(deser.read_u8());
        switch (type)
        {
        case LiketrainResponseType::Invalid:
            break;
        case LiketrainResponseType::DebugMessage:
            data.debug_message.len = deser.read_u32();

            data.debug_message.message = new char[data.debug_message.len];
            deser.read_bytes(reinterpret_cast<uint8_t *>(data.debug_message.message), data.debug_message.len);

            break;
        case LiketrainResponseType::Event:
            data.event.event = new LiketrainEvent();
            data.event.event->deserialize(deser);
            break;
        }
    }
};

#endif // RESPONSE_H