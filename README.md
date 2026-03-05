# liketrain

A UI based application to control old, analog model railways.

- [Introduction](#introduction)
- [How it works](#how-it-works)
  - [Track definition - LTT Files](#track-definition---ltt-files)
  - [Arduino/AVR Hardware](#arduinoavr-hardware)
    - [Communication protocol](#communication-protocol)
- [Explore Science 2026](#explore-science-2026)
- [License](#license)

## Introduction

_liketrain_ is a software application designed to control old, analog model railways using modern hardware and software technologies. It provides a user-friendly interface for managing and automating model train layouts, allowing users to create complex track configurations and control their trains with ease.

The application is built using Rust and leverages the power of microcontrollers (like Arduino/AVR) to interface with the physical components of the model railway, such as switches, sensors, and train controllers.

## How it works

### Track definition - LTT Files

The physical track layout is defined using a [DSL](https://en.wikipedia.org/wiki/Domain-specific_language). The given **l**ike**t**rain **t**rack file (LTT) is parsed line by line and converted into a graph representation of the track layout. There are two core directives used in the LTT format.

#### ID types

- section ID: `S<int>`
- switch ID: Any string that is not a reserved keyword (e.g. `none`, `switch`, `back`).

#### Section connection

Lines starting with a sectino ID (`S<int>:`) are used to describe the connection of a section in both directions.

```
S12:    -> S13 | <- switch(M)
```

would describe a section with ID 12, which is directly connected to section 13 when going forward, and connected to a switch (with ID M) when going backward.

The connection options are defined as follows:

- `S<int>`: direct connection to another section with the given ID
- `none`: no connection in this direction (a dead end)
- `switch(<switch_id>)`: connection to a switch, coming from the switch toe.
- `back(<switch_id>, left|right)`: connection to a switch, coming from the switch heel, with the given direction.

#### Switch connection

A lot of tracks also have crossings, where switch heels are directly connected to each other. These are defined as follows:

```
switch(I, right) -> switch(J, left)
```

This would describe a connection between the right heel of switch I and the left heel of switch J. The direction (left/right) is always defined from the perspective of the switch toe.

### Arduino/AVR Hardware

#### Communication protocol

To communicate with Arduino/AVR hardware, _liketrain_ uses a two layer protocol stack (layer 2 and 3) on top of UART (physical layer).

#### Layer 2 - Serial Frames

To ensure reliable communication, _liketrain_ uses a simple, variable length frame format with a start byte and checksum. In general, this protocol uses LSB (little endian) byte ordering.

| 0 - 7         | 8 - 15            | 16 - 23      | 24 - 31       |
| ------------- | ----------------- | ------------ | ------------- |
| 0xAA          | Length[0-7]       | Length[8-15] | Length[16-23] |
| Length[24-31] | Payload[0-Length] | ...          | Checksum      |

#### Layer 3 - `Deser` packets

_liketrain_ implements its own serialization/deserialization functionality ([Deser](crates/liketrain-hardware/src/deser/mod.rs)), which allows for flexible enum based data packing and unpacking. The byte ordering is LSB (little endian) and the payload is variable length, depending on the data being serialized.

| 0-7            | 8-15         | 16-23 | 24-31 |
| -------------- | ------------ | ----- | ----- |
| Packet variant | Payload[0-x] | ...   | ...   |

The different packet types and their variants are defined in the _liketrain-hardware_ crate.

## Explore Science 2026

This project is part of a project that will be showcased at the [Explore Science 2026](https://www.explore-science.info/friedrichshafen/) exhibition in Friedrichshafen. The exhibition will feature a large model railway layout controlled by _liketrain_, demonstrating the capabilities of the software and hardware integration.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for more informations.

---

> [matteolutz.de](https://matteolutz.de) &nbsp;&middot;&nbsp;
> GitHub [@matteolutz](https://github.com/matteolutz) &nbsp;&middot;&nbsp;
> Email [info@matteolutz.de](mailto:info@matteolutz.de)
