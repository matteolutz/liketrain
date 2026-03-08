#include "section.h"

Section::Section(uint8_t section_id, SectionPowerRelais relais, uint8_t train_detection_pin)
    : section_id(section_id), power_relais(relais), train_detection_pin(train_detection_pin)
{
}

void Section::init()
{
    power_relais.init();
    pinMode(train_detection_pin, INPUT);
}

void Section::update(Queue<LiketrainEvent> &events)
{
    bool occupied = digitalRead(train_detection_pin) == HIGH;

    if (occupied == is_occupied)
        return;

    is_occupied = occupied;

    if (is_occupied)
    {
        auto event = LiketrainEvent::section_event(section_id, SectionEventType::Occupied);
        events.enqueue(event);
    }
    else
    {
        auto event = LiketrainEvent::section_event(section_id, SectionEventType::Freed);
        events.enqueue(event);
    }
}