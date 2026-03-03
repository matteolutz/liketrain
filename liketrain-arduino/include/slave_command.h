#ifndef SLAVE_COMMAND_H
#define SLAVE_COMMAND_H

#include "deser.h"
#include "command.h"

enum class LiketrainSlaveCommandType : uint8_t
{
    Invalid = 0x0,

    EventPoll = 0x1,

    Command = 0x2,
};

union LiketrainSlaveCommandData
{
    struct
    {
        uint32_t slave_id;
    } event_poll;

    struct
    {
        LiketrainCommand *command;
    } command;
};

class LiketrainSlaveCommand : public DeserInterface
{
public:
    LiketrainSlaveCommandType type = LiketrainSlaveCommandType::Invalid;
    LiketrainSlaveCommandData data = {0};

    virtual ~LiketrainSlaveCommand()
    {
        switch (type)
        {
        case LiketrainSlaveCommandType::Command:
            delete data.command.command;
            break;
        }
    }

public:
    static LiketrainSlaveCommand event_poll(uint32_t slave_id)
    {
        LiketrainSlaveCommand command;
        command.type = LiketrainSlaveCommandType::EventPoll;
        command.data.event_poll.slave_id = slave_id;
        return command;
    }

    static LiketrainSlaveCommand command(LiketrainCommand &command)
    {
        LiketrainSlaveCommand slave_command;
        slave_command.type = LiketrainSlaveCommandType::Command;
        slave_command.data.command.command = new LiketrainCommand(command);
        return slave_command;
    }

public:
    void serialize(DeserSerializer &ser) const override
    {
        ser.write_u8(static_cast<uint8_t>(type));

        switch (type)
        {
        case LiketrainSlaveCommandType::Invalid:
            break;
        case LiketrainSlaveCommandType::EventPoll:
            ser.write_u32(data.event_poll.slave_id);
            break;
        case LiketrainSlaveCommandType::Command:
            data.command.command->serialize(ser);
            break;
        }
    }

    void deserialize(DeserDeserializer &deser) override
    {
        type = static_cast<LiketrainSlaveCommandType>(deser.read_u8());
        switch (type)
        {
        case LiketrainSlaveCommandType::Invalid:
            break;
        case LiketrainSlaveCommandType::EventPoll:
            data.event_poll.slave_id = deser.read_u32();
            break;
        case LiketrainSlaveCommandType::Command:
            data.command.command = new LiketrainCommand();
            data.command.command->deserialize(deser);
            break;
        }
    }
};

#endif // SLAVE_COMMAND_H