#include "switch.h"

void SwitchMaster::init()
{
    master_relais.init();
}

void SwitchMaster::arm()
{
    toggle_state = SwitchMasterToggleState::ShouldToggle;
}

void SwitchMaster::update()
{
    switch (toggle_state)
    {
    case SwitchMasterToggleState::Idle:
        break;
    case SwitchMasterToggleState::ShouldToggle:
        master_relais.on();
        last_toggle_time = millis();
        toggle_state = SwitchMasterToggleState::Toggling;
        break;
    case SwitchMasterToggleState::Toggling:
        if (millis() - last_toggle_time >= SWITCH_MASTER_TOGGLE_TIME)
        {
            master_relais.off();
            toggle_state = SwitchMasterToggleState::Idle;
        }
        break;
    }
}