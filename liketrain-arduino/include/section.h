#ifndef SECTION_H
#define SECTION_H

#include <Arduino.h>

#include "relais.h"
#include "queue.h"
#include "event.h"
#include "section_power.h"

#include "ACS712Detector.h"

// the delay in ms, when switching from one power level to anohter
// (not used when switching to or from Off)
#define SECTION_POWER_RELAIS_SWITCHING_DELAY 10

enum class SectionPowerRelaisSwitchingState
{
    Idle,
    DelayedSwitching
};

class SectionPowerRelais
{
private:
    Relais relais[4];
    SectionPower current_power = SectionPower::Off;

    SectionPowerRelaisSwitchingState switching_state = SectionPowerRelaisSwitchingState::Idle;
    SectionPower switching_target_power = SectionPower::Off;
    unsigned long switching_start_time = 0;

    void reset_switching_state()
    {
        switching_state = SectionPowerRelaisSwitchingState::Idle;
        switching_target_power = SectionPower::Off;
        switching_start_time = 0;
    }

private:
    Relais *power_to_relais(SectionPower power)
    {
        switch (power)
        {
        case SectionPower::Off:
            return nullptr;
        case SectionPower::Quarter:
            return &relais[0];
        case SectionPower::Half:
            return &relais[1];
        case SectionPower::ThreeQuarters:
            return &relais[2];
        case SectionPower::Full:
            return &relais[3];
        }

        return nullptr;
    }

public:
    SectionPowerRelais(Relais relais_a, Relais relais_b, Relais relais_c, Relais relais_d) : relais{relais_a, relais_b, relais_c, relais_d} {}

    void init()
    {
        for (size_t i = 0; i < 4; i++)
        {
            relais[i].init();
        }
    }

    void update()
    {
        if (switching_state == SectionPowerRelaisSwitchingState::Idle)
            return; // we are not in the middle of switching, nothing to do

        unsigned long now = millis();

        if (now - switching_start_time < SECTION_POWER_RELAIS_SWITCHING_DELAY)
            return; // we are still in the delay period, wait a bit more

        // power on the new relais
        if (switching_target_power != SectionPower::Off)
            power_to_relais(switching_target_power)->on(); // this shouldn't happen, when switching to off, we can do it directly
        current_power = switching_target_power;

        reset_switching_state();
    }

    SectionPower get_current_power() const { return current_power; }

    void set_power_blocking(SectionPower power)
    {
        if (power == current_power)
            return;

        // cancel the current switching to prevent any race conditions
        reset_switching_state();

        // when switching from off to any other power level, we can directly power on the new relais without delay
        if (current_power == SectionPower::Off)
        {
            power_to_relais(power)->on(); // power can't be Off here, becuase in that case power == current_power should have been true
            current_power = power;
            return;
        }

        // power off the current relais
        power_to_relais(current_power)->off();

        // when we are switching to off, we are done
        if (power == SectionPower::Off)
        {
            current_power = power;
            return;
        }

        delay(SECTION_POWER_RELAIS_SWITCHING_DELAY);

        // power on the new relais
        power_to_relais(power)->on();
        current_power = power;
    }

    void set_power(SectionPower power)
    {
        if (power == current_power)
            return;

        // cancel the current switching to prevent any race conditions
        reset_switching_state();

        // when switching from off to any other power level, we can directly power on the new relais without delay
        if (current_power == SectionPower::Off)
        {
            power_to_relais(power)->on();
            current_power = power;
            return;
        }

        // power off the current relais
        power_to_relais(current_power)->off();

        // when we are switching to off, we are done
        if (power == SectionPower::Off)
        {
            current_power = power;
            return;
        }

        // the only remaining case is switching from one power level to another,
        // in which case we need to delay the switching
        switching_state = SectionPowerRelaisSwitchingState::DelayedSwitching;
        switching_target_power = power;
        switching_start_time = millis();
    }
};

class Section
{
private:
    uint8_t section_id;

    SectionPowerRelais power_relais;
    ACS712Detector train_detection;

    void update_train_detection(Queue<LiketrainEvent> &events);

public:
    Section(uint8_t section_id, SectionPowerRelais relais, ACS712Detector train_detection);

    void init();

    SectionPower current_power() const { return power_relais.get_current_power(); }
    void set_power(SectionPower power) { power_relais.set_power(power); }
    void set_power_blocking(SectionPower power) { power_relais.set_power_blocking(power); }

    ACS712Detector &get_train_detection() { return train_detection; }

    uint8_t id() const { return section_id; }

    void reset()
    {
        set_power(SectionPower::Off);

        // not resetting the train_detection would cause a SectionFree event
        // being enqueued when the next ACS value is read
        train_detection.reset();
    }

    void update(Queue<LiketrainEvent> &events);
};

#endif // SECTION_H