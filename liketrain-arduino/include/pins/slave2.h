#ifndef PINS_H
#define PINS_H

#include "../section.h"
#include "../switch.h"

/* -------------------------------- Sections -------------------------------- */
Section section24(
    24,
    SectionPowerRelais(3, 52, 50, 58),
    A15);
Section section5(
    5,
    SectionPowerRelais(46, 44, 42, 40),
    A14);
Section section21(
    21,
    SectionPowerRelais(38, 36, 34, 32),
    A13);
Section section22(
    22,
    SectionPowerRelais(30, 28, 26, 24),
    A12);
Section section10(
    10,
    SectionPowerRelais(22, 53, 51, 45),
    A11);
Section section6(
    6,
    SectionPowerRelais(47, 45, 43, 41),
    A9);
Section section7(
    7,
    SectionPowerRelais(39, 37, 35, 33),
    A8);
Section section11(
    11,
    SectionPowerRelais(31, 29, 27, 25),
    A10);

Section sections[] = {
    section24,
    section5,
    section21,
    section22,
    section10,
    section6,
    section7,
    section11};

/* -------------------------------- Switches -------------------------------- */
Switch switchK(
    "K",
    Relais(7));
Switch swtichA(
    "A",
    Relais(6));
Switch switchN(
    "N",
    Relais(5));
Switch switchO(
    "4",
    Relais(4));
Switch swtichG(
    "G",
    Relais(2));

Switch switches[] = {
    switchK,
    swtichA,
    switchN,
    switchO,
    swtichG};

#endif // PINS_H