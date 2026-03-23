#ifndef PINS_H
#define PINS_H

#include "../section.h"
#include "../switch.h"

/* -------------------------------- RS485 -------------------------------- */
#define RS485_SERIAL Serial1
#define RS485_DE_PIN 16


/* -------------------------------- Sections -------------------------------- */
Section section20(
    20,
    SectionPowerRelais(Relais(26, LOW), Relais(28, LOW), Relais(30, LOW), Relais(20, LOW)),
    A14);
Section section3(
    3,
    SectionPowerRelais(Relais(34, LOW), Relais(36, LOW), Relais(38, LOW), Relais(40, LOW)),
    A15);
Section section16(
    16,
    SectionPowerRelais(Relais(42, LOW), Relais(44, LOW), Relais(46, LOW), Relais(48, LOW)),
    A13);
Section section15(
    15,
    SectionPowerRelais(Relais(50, LOW), Relais(52, LOW), Relais(53, LOW), Relais(51, LOW)),
    A11);
Section section8(
    8,
    SectionPowerRelais(Relais(49, LOW), Relais(47, LOW), Relais(45, LOW), Relais(43, LOW)),
    A10);
Section section2(
    2,
    SectionPowerRelais(Relais(41, LOW), Relais(39, LOW), Relais(37, LOW), Relais(35, LOW)),
    A9);
Section section14(
    14,
    SectionPowerRelais(Relais(33, LOW), Relais(31, LOW), Relais(29, LOW), Relais(27, LOW)),
    A12);
Section section13(
    13,
    SectionPowerRelais(Relais(25, LOW), Relais(23, LOW), Relais(14, LOW), Relais(15, LOW)),
    A8);

Section *sections[] = {
    &section20,
    &section3,
    &section16,
    &section15,
    &section8,
    &section2,
    &section14,
    &section13};

/* -------------------------------- Switches -------------------------------- */
Switch switchH(
    "H",
    Relais(2));
Switch switchQ(
    "Q",
    Relais(3));
Switch switchF(
    "F",
    Relais(4));
Switch switchE(
    "E",
    Relais(5));
Switch swtichI(
    "I",
    Relais(6, LOW));
Switch switchO(
    "O",
    Relais(7));
Switch switchL(
    "L",
    Relais(24));

Switch* switches[] = {
    &switchH,
    &switchQ,
    &switchF,
    &switchE,
    &swtichI,
    &switchO,
    &switchL};

SwitchMaster switch_master(Relais(22));

#endif // PINS_H