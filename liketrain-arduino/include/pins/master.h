#ifndef PINS_H
#define PINS_H

#include "../section.h"
#include "../switch.h"

/* -------------------------------- Sections -------------------------------- */
Section section20(
    20,
    SectionPowerRelais(26, 28, 30, 20),
    A14
);
Section section3(
    3,
    SectionPowerRelais(34, 36, 38, 40),
    A15
);
Section section16(
    16,
    SectionPowerRelais(42, 44, 46, 48),
    A13
);
Section section15(
    15,
    SectionPowerRelais(50, 52, 53, 51),
    A11
);
Section section8(
    8,
    SectionPowerRelais(49, 47, 45, 43),
    A10
);
Section section2(
    2,
    SectionPowerRelais(41, 39, 37, 35),
    A9
);
Section section14(
    14,
    SectionPowerRelais(33, 31, 29, 27),
    A12
);
Section section13(
    13,
    SectionPowerRelais(25, 23, 14, 15),
    A8
);

Section sections[] = {
    section20,
    section3,
    section16,
    section15,
    section8,
    section2,
    section14,
    section13
};

/* -------------------------------- Switches -------------------------------- */
Switch switchH(
    "H",
    Relais(2)
);
Switch swtichQ(
    "Q",
    Relais(3)
);
Switch switchF(
    "F",
    Relais(4)
);
Switch switchE(
    "E",
    Relais(5)
);
Switch swtichI(
    "I",
    Relais(6)
);
Switch switchO(
    "O",
    Relais(7)
);
Switch switchL(
    "L",
    Relais(24)
);

Switch switches[] = {
    switchH,
    swtichQ,
    switchF,
    switchE,
    swtichI,
    switchO,
    switchL
};

SwitchMaster switch_master(Relais(22));

#endif // PINS_H