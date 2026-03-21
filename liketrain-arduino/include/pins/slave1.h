#ifndef PINS_H
#define PINS_H

#include "../section.h"
#include "../switch.h"

/* -------------------------------- RS485 -------------------------------- */
#define RS485_SERIAL Serial1
#define RS485_DE_PIN 23


/* -------------------------------- Sections -------------------------------- */
Section section12(
    12,
    SectionPowerRelais(28, 26, 24, 22),
    A8);
Section section1(
    1,
    SectionPowerRelais(30, 32, 36, 38),
    A9);
Section section9(
    9,
    SectionPowerRelais(40, 44, 46, 48),
    A13);
Section section4(
    4,
    SectionPowerRelais(50, 51, 49, 47),
    A10);
Section section23(
    23,
    SectionPowerRelais(53, 52, 45, 43),
    A14);
Section section25(
    25,
    SectionPowerRelais(41, 39, 37, 35),
    A12);
Section section26(
    26,
    SectionPowerRelais(33, 31, 29, 27),
    A11);

Section *sections[] = {
    &section12,
    &section1,
    &section9,
    &section4,
    &section23,
    &section25,
    &section26};

/* -------------------------------- Switches -------------------------------- */
Switch switchC(
    "C",
    Relais(2));
Switch swtichM(
    "M",
    Relais(3));
Switch switchB(
    "B",
    Relais(4));
Switch switchP(
    "P",
    Relais(5));
Switch swtichR(
    "R",
    Relais(6));
Switch switchJ(
    "J",
    Relais(7));

Switch switches[] = {
    switchC,
    swtichM,
    switchB,
    switchP,
    swtichR,
    switchJ};

#endif // PINS_H