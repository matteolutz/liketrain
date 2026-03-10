#ifndef EVENT_H
#define EVENT_H

#include "deser.h"
#include "switch.h"

enum class SectionEventType : uint8_t
{
    Occupied = 0,
    Freed = 1
};

struct SectionEvent
{
    uint32_t section_id;
    SectionEventType event_type;
};

enum class LiketrainEventType : uint8_t
{
    Invalid = 0x0,

    Pong = 0x1,
    SectionEvent = 0x2,
    SwitchStateChange = 0x3,
    SectionPowerChange = 0x4,

    Slaves = 0x10
};

union LiketrainEventData
{
    struct
    {
        uint32_t slave_id;
        uint32_t seq;
    } pong;

    SectionEvent section_event;

    struct
    {
        SwitchId swtich_id;
        SwitchState state;
    } switch_state_change;

    struct
    {
        uint32_t section_id;
        SectionPower power;
    } section_power_change;

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

    static LiketrainEvent section_event(uint32_t section_id, SectionEventType event_type)
    {
        LiketrainEvent event;
        event.type = LiketrainEventType::SectionEvent;
        event.data.section_event.section_id = section_id;
        event.data.section_event.event_type = event_type;
        return event;
    }

    static LiketrainEvent section_event(uint32_t section_id, SectionPower power)
    {
        LiketrainEvent event;
        event.type = LiketrainEventType::SectionPowerChange;
        event.data.section_power_change.section_id = section_id;
        event.data.section_power_change.power = power;
        return event;
    }

    static LiketrainEvent switch_state_change(SwitchId switch_id, SwitchState state)
    {
        LiketrainEvent event;
        event.type = LiketrainEventType::SwitchStateChange;
        memcpy(event.data.switch_state_change.swtich_id, switch_id, sizeof(SwitchId));
        event.data.switch_state_change.state = state;
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
        case LiketrainEventType::Invalid:
            break;

        case LiketrainEventType::Pong:
            ser.write_u32(data.pong.slave_id);
            ser.write_u32(data.pong.seq);
            break;
        case LiketrainEventType::SectionEvent:
            ser.write(data.section_event);
            break;
        case LiketrainEventType::SwitchStateChange:
            ser.write(data.switch_state_change.swtich_id);
            ser.write_u8(static_cast<uint8_t>(data.switch_state_change.state));
            break;
        case LiketrainEventType::SectionPowerChange:
            ser.write_u32(data.section_power_change.section_id);
            ser.write_u8(static_cast<uint8_t>(data.section_power_change.power));
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
        case LiketrainEventType::Invalid:
            break;

        case LiketrainEventType::Pong:
            data.pong.slave_id = deser.read_u32();
            data.pong.seq = deser.read_u32();
            break;
        case LiketrainEventType::SectionEvent:
            deser.read(data.section_event);
            break;
        case LiketrainEventType::SwitchStateChange:
            deser.read(data.switch_state_change.swtich_id);
            data.switch_state_change.state = static_cast<SwitchState>(deser.read_u8());
            break;
        case LiketrainEventType::SectionPowerChange:
            data.section_power_change.section_id = deser.read_u32();
            data.section_power_change.power = static_cast<SectionPower>(deser.read_u8());
            break;
        case LiketrainEventType::Slaves:
            data.slaves.n_slaves = deser.read_u32();
            break;
        }
    }
};

#endif // EVENT_H