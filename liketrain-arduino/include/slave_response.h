#ifndef SLAVE_RESPONSE_H
#define SLAVE_RESPONSE_H

#include "deser.h"
#include "event.h"

enum class LiketrainSlaveResponseType : uint8_t
{
    Invalid = 0x0,

    EventCount = 0x1,
    Event = 0x2,
};

union LiketrainSlaveResponseData
{
    struct
    {
        uint32_t event_count;
    } event_count;

    struct
    {
        LiketrainEvent *event;
    } event;
};

class LiketrainSlaveResponse : public DeserInterface
{
public:
    LiketrainSlaveResponseType type = LiketrainSlaveResponseType::Invalid;
    LiketrainSlaveResponseData data = {0};

    virtual ~LiketrainSlaveResponse()
    {
        switch (type)
        {
        case LiketrainSlaveResponseType::Event:
            delete data.event.event;
            break;
        default:
            break;
        }
    }

public:
    static LiketrainSlaveResponse event_count(uint32_t event_count)
    {
        LiketrainSlaveResponse response;
        response.type = LiketrainSlaveResponseType::EventCount;
        response.data.event_count.event_count = event_count;
        return response;
    }

    static LiketrainSlaveResponse event(LiketrainEvent &event)
    {
        LiketrainSlaveResponse response;
        response.type = LiketrainSlaveResponseType::Event;
        response.data.event.event = new LiketrainEvent(event);
        return response;
    }

public:
    void serialize(DeserSerializer &ser) const override
    {
        ser.write_u8(static_cast<uint8_t>(type));

        switch (type)
        {
        case LiketrainSlaveResponseType::Invalid:
            break;
        case LiketrainSlaveResponseType::EventCount:
            ser.write_u32(data.event_count.event_count);
            break;
        case LiketrainSlaveResponseType::Event:
            data.event.event->serialize(ser);
            break;
        }
    }

    void deserialize(DeserDeserializer &deser) override
    {
        type = static_cast<LiketrainSlaveResponseType>(deser.read_u8());
        switch (type)
        {
        case LiketrainSlaveResponseType::Invalid:
            break;
        case LiketrainSlaveResponseType::EventCount:
            data.event_count.event_count = deser.read_u32();
            break;
        case LiketrainSlaveResponseType::Event:
            data.event.event = new LiketrainEvent();
            data.event.event->deserialize(deser);
            break;
        }
    }
};

#endif // SLAVE_RESPONSE_H