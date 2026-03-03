#ifndef CONFIG_H
#define CONFIG_H

#include "slave.h"

// #define SLAVE_ID 1
#define SLAVE_COUNT 1

#ifdef SLAVE_ID
#if SLAVE_ID == 0
#error "SLAVE_ID cannot be 0, as that is reserved for the master"
#endif // SLAVE_ID == 0
#define IS_SLAVE
SlaveId slave_id = SLAVE_ID;
#else
#ifndef SLAVE_COUNT
#error "SLAVE_COUNT must be defined if SLAVE_ID is not defined"
#endif

#define IS_MASTER
SlaveId slave_id = SlaveId::master();
#endif // SLAVE_ID

#endif // CONFIG_H