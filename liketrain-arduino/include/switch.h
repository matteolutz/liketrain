#ifndef SWITCH_H
#define SWITCH_H

#include <Arduino.h>

#include "relais.h"

#define SWITCH_ID_LEN 32
using SwitchId = uint8_t[SWITCH_ID_LEN];

enum class SwitchState : uint8_t
{
    Left = 0,
    Right = 1
};

#define SWITCH_MASTER_TOGGLE_TIME 200 // ms to toggle the master switch when changing state

class Switch
{
private:
    SwitchId switch_id;
    Relais relais;

public:
    Switch(SwitchId switch_id, Relais relais) : relais(relais)
    {
        memccpy(this->switch_id, switch_id, 0, SWITCH_ID_LEN);
    }

    Switch(const char *switch_id_str, Relais relais) : relais(relais)
    {
        strncpy((char *)this->switch_id, switch_id_str, SWITCH_ID_LEN);
    }

    void init() { relais.init(); }

    inline const SwitchId *id() const { return &switch_id; }
    inline bool matches_id(const SwitchId &other_id) const { return memcmp(switch_id, other_id, SWITCH_ID_LEN) == 0; }

    inline void set_state(SwitchState state)
    {
        switch (state)
        {
        case SwitchState::Left:
            relais.off();
            break;
        case SwitchState::Right:
            relais.on();
            break;
        }
    }

    inline void reset() { set_state(SwitchState::Left); }
};

enum class SwitchMasterToggleState : uint8_t
{
    Idle,
    ShouldToggle,
    Toggling
};

class SwitchMaster
{
private:
    Relais master_relais;

    SwitchMasterToggleState toggle_state = SwitchMasterToggleState::Idle;
    unsigned long last_toggle_time = 0;

public:
    SwitchMaster(Relais master_relais) : master_relais(master_relais) {};

    void init();

    /// @brief Arm the switch to toggle when toggle is called
    void arm();

    /// @brief Toggle the switch if it is armed, and disarm it. Call this in the main loop
    void update();

    void blocking_toggle();
};

#endif // SWITCH_H