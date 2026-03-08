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

#define SWITCH_MASTER_TOGGLE_TIME 100 // ms to toggle the master switch when changing state

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
};

#endif // SWITCH_H