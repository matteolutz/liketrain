#ifndef EVENT_H
#define EVENT_H

#include "deser.h"

enum class LiketrainEventType : uint8_t
{
    Invalid = 0x0,

    Pong = 0x1,
    SectionEvent = 0x2,
    SwitchStateChange = 0x3,

    Slaves = 0x10
};

union LiketrainEventData
{
    struct
    {
        uint32_t slave_id;
        uint32_t seq;
    } pong;

    struct
    {
        uint32_t n_slaves;
    } slaves;
};

class LiketrainEvent : public DeserInterface
{
public:
    LiketrainEventType type = LiketrainEventType::Invalid;
    LiketrainEventData data = {0};

    virtual ~LiketrainEvent() {}

public:
    static LiketrainEvent pong(uint32_t slave_id, uint32_t seq)
    {
        LiketrainEvent event;
        event.type = LiketrainEventType::Pong;
        event.data.pong.slave_id = slave_id;
        event.data.pong.seq = seq;
        return event;
    }

    static LiketrainEvent slaves(uint32_t n_slaves)
    {
        LiketrainEvent event;
        event.type = LiketrainEventType::Slaves;
        event.data.slaves.n_slaves = n_slaves;
        return event;
    }

public:
    void serialize(DeserSerializer &ser) const override
    {
        ser.write_u8(static_cast<uint8_t>(type));

        switch (type)
        {
        case LiketrainEventType::Pong:
            ser.write_u32(data.pong.slave_id);
            ser.write_u32(data.pong.seq);
            break;
        case LiketrainEventType::Slaves:
            ser.write_u32(data.slaves.n_slaves);
            break;
        }
    }

    void deserialize(DeserDeserializer &deser) override
    {
        type = static_cast<LiketrainEventType>(deser.read_u8());
        switch (type)
        {
        case LiketrainEventType::Pong:
            data.pong.slave_id = deser.read_u32();
            data.pong.seq = deser.read_u32();
            break;
        case LiketrainEventType::Slaves:
            data.slaves.n_slaves = deser.read_u32();
            break;
        }
    }
};

#endif // EVENT_H