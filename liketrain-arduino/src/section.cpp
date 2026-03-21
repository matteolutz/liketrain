#include "section.h"

Section::Section(uint8_t section_id, SectionPowerRelais relais, ACS712Detector train_detection)
    : section_id(section_id), power_relais(relais), train_detection(train_detection)
{
}

void Section::init()
{
    power_relais.init();

    train_detection.begin();
}

void Section::update_train_detection(Queue<LiketrainEvent> &events)
{
    ACS712DetectorEvent event = train_detection.update(); // get a new sample for the train detection sensor

    switch (event)
    {
    case ACS712DetectorEvent::None:
        break; // no change, do nothing
    case ACS712DetectorEvent::Occupied:
    {
        auto event = LiketrainEvent::section_event(section_id, SectionEventType::Occupied);
        events.enqueue(event);

        break;
    }
    case ACS712DetectorEvent::Free:
    {
        auto event = LiketrainEvent::section_event(section_id, SectionEventType::Freed);
        events.enqueue(event);
        break;
    }
    }
}

void Section::update(Queue<LiketrainEvent> &events)
{
    update_train_detection(events);

    power_relais.update(); // update the relais (necessary if we are doing a non-blocking delayed power change)
}