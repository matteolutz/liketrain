#ifndef CONFIG_H
#define CONFIG_H

// This file contains the compile configuration for uploading this code to the chip.
//     Master   -> Arduino 3
//     Slave 1  -> Arduino 2
//     Slave 2  -> Arduino 1

#include "slave.h"

#define SLAVE_ID 1
#define SLAVE_COUNT 2

// #define SWITCH_TEST

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

// include correct pin definition
#ifdef IS_MASTER
#include "pins/master.h"
#elif SLAVE_ID == 1
#include "pins/slave1.h"
#elif SLAVE_ID == 2
#include "pins/slave2.h"
#endif

#endif // CONFIG_H