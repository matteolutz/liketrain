#ifndef MODE_H
#define MODE_H

#include <Arduino.h>
#include "panic.h"

#define SLAVE_MASTER_ID 0

class SlaveId
{
private:
    uint32_t id;

public:
    SlaveId(uint32_t id) : id(id) {};

    static SlaveId master()
    {
        return SlaveId(SLAVE_MASTER_ID);
    };

    static SlaveId slave(uint32_t id)
    {
        if (id == SLAVE_MASTER_ID)
        {
            panic("Cannot use master id as slave id");
        }

        return SlaveId(id);
    }

    uint32_t get() const
    {
        return id;
    }

    bool is_master() const
    {
        return id == SLAVE_MASTER_ID;
    };
};

#endif // MODE_H