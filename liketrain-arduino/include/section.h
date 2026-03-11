#ifndef SECTION_H
#define SECTION_H

#include <Arduino.h>

#include "relais.h"
#include "queue.h"
#include "event.h"
#include "section_power.h"

#define SECTION_POWER_RELAIS_SWITCHING_DELAY 10

class SectionPowerRelais
{
private:
    Relais relais[4];
    SectionPower current_power = SectionPower::Off;

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

    SectionPower get_current_power() const { return current_power; }

    void set_power(SectionPower power)
    {
        if (power == current_power)
            return;

        auto previous_power = current_power;
        current_power = power;

        if (previous_power != SectionPower::Off)
        {
            // we need to turn of the previous power level relais
            power_to_relais(previous_power)->off();

            // if we need to power up a different relais, delay
            if (power != SectionPower::Off)
            {
                delay(SECTION_POWER_RELAIS_SWITCHING_DELAY);
            }
        }

        if (power != SectionPower::Off)
        {
            power_to_relais(power)->on();
        }
    }
};

class Section
{
private:
    uint8_t section_id;

    SectionPowerRelais power_relais;
    uint8_t train_detection_pin;

    bool is_occupied = false;

public:
    Section(uint8_t section_id, SectionPowerRelais relais, uint8_t train_detection_pin);

    void init();

    bool occupied() const { return is_occupied; }

    SectionPower current_power() const { return power_relais.get_current_power(); }
    void set_power(SectionPower power) { power_relais.set_power(power); }

    void reset()
    {
        set_power(SectionPower::Off);
    }

    void update(Queue<LiketrainEvent> &events);
};

#endif // SECTION_H