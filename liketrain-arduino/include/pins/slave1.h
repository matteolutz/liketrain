#ifndef PINS_H
#define PINS_H

#include "../section.h"
#include "../switch.h"

/* -------------------------------- RS485 -------------------------------- */
#define RS485_SERIAL Serial3
#define RS485_DE_PIN 23


/* -------------------------------- Sections -------------------------------- */
Section section12(
    12,
    SectionPowerRelais(Relais(28, LOW), Relais(26, LOW), Relais(24, LOW), Relais(22, LOW)),
    A8);
Section section1(
    1,
    SectionPowerRelais(Relais(30, LOW), Relais(32, LOW), Relais(36, LOW), Relais(38, LOW)),
    A9);
Section section9(
    9,
    SectionPowerRelais(Relais(40, LOW), Relais(44, LOW), Relais(46, LOW), Relais(48, LOW)),
    A13);
Section section4(
    4,
    SectionPowerRelais(Relais(50, LOW), Relais(51, LOW), Relais(49, LOW), Relais(47, LOW)),
    A10);
Section section23(
    23,
    SectionPowerRelais(Relais(53, LOW), Relais(52, LOW), Relais(19, LOW), Relais(43, LOW)),
    A14);
Section section25(
    25,
    SectionPowerRelais(Relais(41, LOW), Relais(39, LOW), Relais(37, LOW), Relais(35, LOW)),
    A12);
Section section26(
    26,
    SectionPowerRelais(Relais(33, LOW), Relais(31, LOW), Relais(29, LOW), Relais(27, LOW)),
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

Switch *switches[] = {
    &switchC,
    &swtichM,
    &switchB,
    &switchP,
    &swtichR,
    &switchJ};

#endif // PINS_H