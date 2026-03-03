#ifndef SECTION_H
#define SECTION_H

#include <Arduino.h>

#include "queue.h"
#include "event.h"

#define SECTION_POWER_RELAIS_SWITCHING_DELAY 10

enum class SectionPower : uint8_t
{
    Off = 0,
    Quarter = 1,
    Half = 2,
    ThreeQuarters = 3,
    Full = 4
};

class SectionPowerRelais
{
private:
    uint8_t relais[4];
    SectionPower current_power = SectionPower::Off;

private:
    size_t power_to_relais_idx(SectionPower power)
    {
        switch (power)
        {
        case SectionPower::Off:
            return -1;
        case SectionPower::Quarter:
            return 0;
        case SectionPower::Half:
            return 1;
        case SectionPower::ThreeQuarters:
            return 2;
        case SectionPower::Full:
            return 3;
        }
    }

public:
    SectionPowerRelais(uint8_t relais1, uint8_t relais2, uint8_t relais3, uint8_t relais4)
    {
        relais[0] = relais1;
        relais[1] = relais2;
        relais[2] = relais3;
        relais[3] = relais4;

        for (int i = 0; i < 4; i++)
        {
            pinMode(relais[i], OUTPUT);
            digitalWrite(relais[i], LOW);
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
            digitalWrite(relais[power_to_relais_idx(previous_power)], LOW);

            // if we need to power up a different relais, delay
            if (power != SectionPower::Off)
            {
                delay(SECTION_POWER_RELAIS_SWITCHING_DELAY);
            }
        }

        if (power != SectionPower::Off)
        {
            digitalWrite(relais[power_to_relais_idx(power)], HIGH);
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