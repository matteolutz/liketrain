#include "section.h"

Section::Section(uint8_t section_id, SectionPowerRelais relais, ACS712 train_detection)
    : section_id(section_id), power_relais(relais), train_detection(train_detection)
{
}

void Section::init()
{
    power_relais.init();
    train_detection.begin();
}

void Section::update(Queue<LiketrainEvent> &events)
{
    train_detection.update();

    // no rms value available yet, skip
    if (!train_detection.available())
        return;

    bool occupied = train_detection.get_rms() > 0.1;

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