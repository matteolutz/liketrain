#ifndef SLAVE_BUS_MESSAGE_H
#define SLAVE_BUS_MESSAGE_H

#include "deser.h"
#include "command.h"
#include "event.h"

enum class LiketrainSlaveBusMessageType : uint8_t
{
    Invalid = 0x0,

    SlaveEventCount = 0x1,
    SlaveEvent = 0x2,

    MasterEventPoll = 0x10,
    MasterCommand = 0x11,
};

union LiketrainSlaveBusMessageData
{
    struct
    {
        uint32_t event_count;
    } slave_event_count;

    struct
    {
        LiketrainEvent *event;
    } slave_event;

    struct
    {
        uint32_t slave_id;
    } master_event_poll;

    struct
    {
        LiketrainCommand *command;
    } master_command;
};

class LiketrainSlaveBusMessage : public DeserInterface
{
public:
    LiketrainSlaveBusMessageType type = LiketrainSlaveBusMessageType::Invalid;
    LiketrainSlaveBusMessageData data = {0};

    virtual ~LiketrainSlaveBusMessage()
    {
        switch (type)
        {
        case LiketrainSlaveBusMessageType::SlaveEvent:
            delete data.slave_event.event;
            break;
        case LiketrainSlaveBusMessageType::MasterCommand:
            delete data.master_command.command;
            break;
        default:
            break;
        }
    }

public:
    static LiketrainSlaveBusMessage slave_event_count(uint32_t event_count)
    {
        LiketrainSlaveBusMessage message;
        message.type = LiketrainSlaveBusMessageType::SlaveEventCount;
        message.data.slave_event_count.event_count = event_count;
        return message;
    }

    static LiketrainSlaveBusMessage slave_event(LiketrainEvent &event)
    {
        LiketrainSlaveBusMessage message;
        message.type = LiketrainSlaveBusMessageType::SlaveEvent;
        message.data.slave_event.event = new LiketrainEvent(event);
        return message;
    }

    static LiketrainSlaveBusMessage master_event_poll(uint32_t slave_id)
    {
        LiketrainSlaveBusMessage message;
        message.type = LiketrainSlaveBusMessageType::MasterEventPoll;
        message.data.master_event_poll.slave_id = slave_id;
        return message;
    }

    static LiketrainSlaveBusMessage master_command(LiketrainCommand &command)
    {
        LiketrainSlaveBusMessage message;
        message.type = LiketrainSlaveBusMessageType::MasterCommand;
        message.data.master_command.command = new LiketrainCommand(command);
        return message;
    }

public:
    void serialize(DeserSerializer &ser) const override
    {
        ser.write_u8(static_cast<uint8_t>(type));

        switch (type)
        {
        case LiketrainSlaveBusMessageType::Invalid:
            break;
        case LiketrainSlaveBusMessageType::SlaveEventCount:
            ser.write_u32(data.slave_event_count.event_count);
            break;
        case LiketrainSlaveBusMessageType::SlaveEvent:
            data.slave_event.event->serialize(ser);
            break;
        case LiketrainSlaveBusMessageType::MasterEventPoll:
            ser.write_u32(data.master_event_poll.slave_id);
            break;
        case LiketrainSlaveBusMessageType::MasterCommand:
            data.master_command.command->serialize(ser);
            break;
        }
    }

    void deserialize(DeserDeserializer &deser) override
    {
        type = static_cast<LiketrainSlaveBusMessageType>(deser.read_u8());
        switch (type)
        {
        case LiketrainSlaveBusMessageType::Invalid:
            break;
        case LiketrainSlaveBusMessageType::SlaveEventCount:
            data.slave_event_count.event_count = deser.read_u32();
            break;
        case LiketrainSlaveBusMessageType::SlaveEvent:
            data.slave_event.event = new LiketrainEvent();
            data.slave_event.event->deserialize(deser);
            break;
        case LiketrainSlaveBusMessageType::MasterEventPoll:
            data.master_event_poll.slave_id = deser.read_u32();
            break;
        case LiketrainSlaveBusMessageType::MasterCommand:
            data.master_command.command = new LiketrainCommand();
            data.master_command.command->deserialize(deser);
            break;
        }
    }
};

#endif // SLAVE_BUS_MESSAGE_H