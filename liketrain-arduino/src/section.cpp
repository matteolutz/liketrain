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

void Section::update_train_detection(Queue<LiketrainEvent> &events)
{
    train_detection.update(); // get a new sample for the train detection sensor

    // no rms value available yet, skip
    if (!train_detection.available())
        return;

    bool occupied = train_detection.get_rms() > SECTION_TRAIN_DETECTION_RMS_THRESHOLD;

    if (occupied == is_occupied)
        return; // state hasn't changed

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

void Section::update(Queue<LiketrainEvent> &events)
{
    power_relais.update(); // update the relais (necessary if we are doing a non-blocking delayed power change)

    update_train_detection(events);
}