#ifndef COMMAND_H
#define COMMAND_H

#include "deser.h"
#include "section.h"
#include "switch.h"

enum class LiketrainCommandType : uint8_t
{
    Invalid = 0x0,

    Ping = 0x1,
    GetSlaves = 0x2,

    SetSectionPower = 0x10,
    SetSwitchState = 0x20,

    ResetAll = 0x30
};

union LiketrainCommandData
{
    struct
    {
        uint32_t slave_id;
        uint32_t seq;
    } ping;

    struct
    {
        uint32_t section_id;
        SectionPower power;
    } set_section_power;

    struct
    {
        SwitchId switch_id;
        SwitchState state;
    } set_switch_state;
};

class LiketrainCommand : public DeserInterface
{
public:
    LiketrainCommandType type = LiketrainCommandType::Invalid;
    LiketrainCommandData data = {0};

    virtual ~LiketrainCommand() {}

public:
    void serialize(DeserSerializer &ser) const override
    {
        ser.write_u8(static_cast<uint8_t>(type));
        switch (type)
        {
        case LiketrainCommandType::Invalid:
            break;
        case LiketrainCommandType::Ping:
            ser.write_u32(data.ping.slave_id);
            ser.write_u32(data.ping.seq);
            break;
        case LiketrainCommandType::GetSlaves:
            break;
        case LiketrainCommandType::SetSectionPower:
            ser.write_u32(data.set_section_power.section_id);
            ser.write_u8(static_cast<uint8_t>(data.set_section_power.power));
            break;
        case LiketrainCommandType::SetSwitchState:
            ser.write_bytes(data.set_switch_state.switch_id, sizeof(SwitchId));
            ser.write_u8(static_cast<uint8_t>(data.set_switch_state.state));
            break;
        case LiketrainCommandType::ResetAll:
            break;
        }
    }

    void deserialize(DeserDeserializer &deser) override
    {
        type = static_cast<LiketrainCommandType>(deser.read_u8());

        switch (type)
        {
        case LiketrainCommandType::Invalid:
            break;
        case LiketrainCommandType::Ping:
            data.ping.slave_id = deser.read_u32();
            data.ping.seq = deser.read_u32();
            break;
        case LiketrainCommandType::GetSlaves:
            break;
        case LiketrainCommandType::SetSectionPower:
            data.set_section_power.section_id = deser.read_u32();
            data.set_section_power.power = static_cast<SectionPower>(deser.read_u8());
            break;
        case LiketrainCommandType::SetSwitchState:
            deser.read_bytes(data.set_switch_state.switch_id, sizeof(SwitchId));
            data.set_switch_state.state = static_cast<SwitchState>(deser.read_u8());
            break;
        case LiketrainCommandType::ResetAll:
            break;
        }
    }
};

#endif // COMMAND_H