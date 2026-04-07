#ifndef PINS_H
#define PINS_H

#include "../section.h"
#include "../switch.h"

/* -------------------------------- RS485 -------------------------------- */
#define RS485_SERIAL Serial3
#define RS485_DE_PIN 23


/* -------------------------------- Sections -------------------------------- */
Section section24(
    24,
    SectionPowerRelais(Relais(3, LOW), Relais(52, LOW), Relais(50, LOW), Relais(18, LOW)),
    A15);
Section section5(
    5,
    SectionPowerRelais(Relais(46, LOW), Relais(44, LOW), Relais(42, LOW), Relais(40, LOW)),
    A14);
Section section21(
    21,
    SectionPowerRelais(Relais(38, LOW), Relais(36, LOW), Relais(34, LOW), Relais(32, LOW)),
    A13);
Section section22(
    22,
    SectionPowerRelais(Relais(30, LOW), Relais(28, LOW), Relais(26, LOW), Relais(24, LOW)),
    A12);
Section section10(
    10,
    SectionPowerRelais(Relais(22, LOW), Relais(53, LOW), Relais(51, LOW), Relais(49, LOW)),
    A11);
Section section6(
    6,
    SectionPowerRelais(Relais(41, LOW), Relais(43, LOW), Relais(45, LOW), Relais(47, LOW)),
    A9);
Section section7(
    7,
    SectionPowerRelais(Relais(33, LOW), Relais(35, LOW), Relais(37, LOW), Relais(39, LOW)),
    A8);
Section section11(
    11,
    SectionPowerRelais(Relais(25, LOW), Relais(27, LOW), Relais(29, LOW), Relais(31, LOW)),
    A10);

Section *sections[] = {
    &section24,
    &section5,
    &section21,
    &section22,
    &section10,
    &section6,
    &section7,
    &section11};

/* -------------------------------- Switches -------------------------------- */
Switch switchK(
    "K",
    Relais(7));
Switch swtichA(
    "A",
    Relais(6));
Switch switchN(
    "N",
    Relais(5, LOW));
Switch switchO(
    "O",
    Relais(4));
Switch swtichG(
    "G",
    Relais(2, LOW));

Switch* switches[] = {
    &switchK,
    &swtichA,
    &switchN,
    &switchO,
    &swtichG};

#endif // PINS_H