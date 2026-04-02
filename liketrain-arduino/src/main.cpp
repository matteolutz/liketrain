#include <Arduino.h>

#include "panic.h"

#include "queue.h"
#include "deser.h"
#include "command.h"
#include "event.h"
#include "response.h"

#include "slave_command.h"
#include "slave_response.h"

#include "config.h"

#include <Wire.h>
#include <LiquidCrystal_I2C.h>

DeserHeapBufferDeserializer deser;
DeserBufferSerializer<128> ser;

RS485 rs485(RS485_SERIAL, RS485_DE_PIN);
DeserSerial rs485_serial(rs485);

Queue<LiketrainEvent> events(64);
Queue<LiketrainCommand> slave_relay(32);

#ifdef IS_MASTER
LiquidCrystal_I2C lcd(0x3F, 16, 2);
unsigned long last_debug = millis();
const unsigned long debug_interval = 100;

DeserSerial usb_serial(Serial);
#endif

// Whether to send an ACK response to the host after processing the next command.
// Set to true when a command is received, and set back to false after sending the ACK.
bool send_ack = false;

// ~~~~~~~~~~~ Master functions ~~~~~~~~~~ //
#ifdef IS_MASTER
void read_host_commands();
void poll_slaves();
void handle_events();
void send_ack_to_host();
#endif

// ~~~~~~~~~~~ Slave functions ~~~~~~~~~~~ //
#ifndef IS_MASTER
void read_master_commands();
#endif

// Handle a received command and return whether it was handled successfully.
// If false is returned, the master will know to relay the command onto the slave bus
bool handle_command(LiketrainCommand &cmd);

void setup()
{
  panic_init(LED_BUILTIN);

  pinMode(LED_BUILTIN, OUTPUT);
  for (int i = 0; i < 3; i++)
  {
    digitalWrite(LED_BUILTIN, HIGH);
    delay(100);
    digitalWrite(LED_BUILTIN, LOW);
    delay(100);
  }

  for (Section *section : sections)
  {
    section->init();
  }

  for (Switch *sw : switches)
  {
    sw->init();
  }

#ifdef IS_MASTER
  lcd.init();
  lcd.backlight();

  switch_master.init();

  usb_serial.init();
#endif

  rs485_serial.init();

  /*
  for (int i = 0; i < 100; i++) {
    section15.set_power_blocking(SectionPower::Quarter);
    delay(1000);

    section15.set_power_blocking(SectionPower::Half);
    delay(1000);

    section15.set_power_blocking(SectionPower::ThreeQuarters);
    delay(1000);

    section15.set_power_blocking(SectionPower::Full);
    delay(1000);

    section15.set_power_blocking(SectionPower::Off);
    delay(1000);
  }*/

  /*
  for (int i = 0; i < 100; i++) {
    switchH.set_state(SwitchState::Left);
    delay(100);
    switch_master.blocking_toggle();
    delay(1000);
    switchH.set_state(SwitchState::Right);
    delay(100);
    switch_master.blocking_toggle();
    delay(2000);
  }
    */

#ifdef SWITCH_TEST
#ifdef IS_MASTER
  delay(2000);

  for (Switch &sw : switches)
  {
    sw.set_state(SwitchState::Left);
    delay(100);
    switch_master.blocking_toggle();
    delay(2000);

    sw.set_state(SwitchState::Right);
    delay(100);
    switch_master.blocking_toggle();
    delay(5000);
  }
#endif
#endif
}

void loop()
{
#ifdef IS_MASTER
  read_host_commands();
#else
  read_master_commands();
#endif

  // update the sections
  /*for (Section *section : sections)
  {
    section->update(events);
  }*/

  section16.update(events);
  section15.update(events);
  section14.update(events);

#if false
#ifdef IS_MASTER
  if (millis() - last_debug >= debug_interval)
  {
    last_debug = millis();
    /*lcd.clear();
    lcd.setCursor(0, 0);
    lcd.print(value);*/

    int peak_value = section16.get_train_detection().get_frame_peak();

    char buffer[32];

    snprintf(buffer, sizeof(buffer), "N: %d", peak_value);
    auto response = LiketrainResponse::debug_message(buffer, strlen(buffer));

    ser.reset();
    response.serialize(ser);

    usb_serial.write_frame(ser);
  }
#endif
#endif

#ifdef IS_MASTER
  poll_slaves();

  // if the switch master relais was armed
  // toggle it, which will cause it to change the switch state
  switch_master.update();
#endif

#ifdef IS_MASTER
  handle_events();
#endif

#ifdef IS_MASTER
  if (send_ack)
  {
    send_ack_to_host();
    send_ack = false;
  }
#endif
}

#ifdef IS_MASTER
void read_host_commands()
{
  usb_serial.update();

  while (usb_serial.read_frame(deser))
  {
    LiketrainCommand cmd;
    cmd.deserialize(deser);

    if (!handle_command(cmd))
    {
      slave_relay.enqueue(cmd);
    }

    send_ack = true;
  }
}

void poll_slaves()
{
  // send unhandled commands to slaves
  LiketrainCommand cmd;
  while (slave_relay.dequeue(cmd))
  {
    auto slave_cmd = LiketrainSlaveCommand::command(cmd);

    ser.reset();
    slave_cmd.serialize(ser);

    rs485_serial.write_frame(ser);
  }

  for (uint32_t slave_id = 1; slave_id <= SLAVE_COUNT; slave_id++)
  {
    auto slave_cmd = LiketrainSlaveCommand::event_poll(slave_id);

    ser.reset();
    slave_cmd.serialize(ser);

    rs485_serial.write_frame(ser);

    if (!rs485_serial.await_frame(deser, 50))
    {
      // timeout
      continue;
    }

    LiketrainSlaveResponse slave_response;
    slave_response.deserialize(deser);

    if (slave_response.type != LiketrainSlaveResponseType::EventCount)
    {
      continue;
    }

    auto event_count = slave_response.data.event_count.event_count;

    lcd.setCursor(0, 0);
    lcd.print("S.Cnt: ");
    lcd.print(event_count);

    // get as many events as the slave has, or until a timeout occurs
    for (uint32_t i = 0; i < event_count; i++)
    {

      if (!rs485_serial.await_frame(deser, 50))
      {
        // timeout
        break;
      }

      LiketrainSlaveResponse slave_event_response;
      slave_event_response.deserialize(deser);
      if (slave_event_response.type != LiketrainSlaveResponseType::Event)
      {
        break;
      }

      events.enqueue(*slave_event_response.data.event.event);
    }
  }
}

void handle_events()
{
  LiketrainEvent evt;
  while (events.dequeue(evt))
  {
    if (evt.type == LiketrainEventType::SwitchStateChange)
    {
      // this will cause the switch master relais to toggle
      // in the next loop iteration
      switch_master.arm();
    }

    auto response = LiketrainResponse::event(evt);

    ser.reset();
    response.serialize(ser);

    usb_serial.write_frame(ser);
  }
}

void send_ack_to_host()
{
  auto response = LiketrainResponse::ack();

  ser.reset();
  response.serialize(ser);

  usb_serial.write_frame(ser);
}

#else
void read_master_commands()
{
  rs485_serial.update();

  while (rs485_serial.read_frame(deser))
  {
    LiketrainSlaveCommand cmd;
    cmd.deserialize(deser);

    switch (cmd.type)
    {
    case LiketrainSlaveCommandType::Invalid:
      // shouldn't happen
      break;
    case LiketrainSlaveCommandType::Command:
      handle_command(*cmd.data.command.command);
      break;
    case LiketrainSlaveCommandType::EventPoll:
      if (slave_id.get() != cmd.data.event_poll.slave_id)
      {
        // not for me
        break;
      }

      auto response = LiketrainSlaveResponse::event_count(events.size());

      ser.reset();
      response.serialize(ser);

      rs485_serial.write_frame(ser);

      LiketrainEvent evt;
      while (events.dequeue(evt))
      {
        auto event_response = LiketrainSlaveResponse::event(evt);
        ser.reset();
        event_response.serialize(ser);

        rs485_serial.write_frame(ser);
      }

      break;
    }
  }
}
#endif

bool handle_command(LiketrainCommand &cmd)
{
  switch (cmd.type)
  {
  case LiketrainCommandType::Invalid:
    break;
  case LiketrainCommandType::Ping:
  {
    if (slave_id.get() != cmd.data.ping.slave_id)
      return false; // not for me

    auto pong_event = LiketrainEvent::pong(cmd.data.ping.slave_id, cmd.data.ping.seq);
    events.enqueue(pong_event);

    return true;
  }
  case LiketrainCommandType::GetSlaves:
  {
    if (!slave_id.is_master())
      return false; // slaves should not receive this command

    auto slave_event = LiketrainEvent::slaves(SLAVE_COUNT);
    events.enqueue(slave_event);

    return true;
  }
  case LiketrainCommandType::SetSectionPower:
  {
    for (Section *section : sections)
    {
      if (section->id() != cmd.data.set_section_power.section_id)
        continue;

      // we found the section
      section->set_power(cmd.data.set_section_power.power);

      events.enqueue(
          LiketrainEvent::section_power_change(
              cmd.data.set_section_power.section_id,
              cmd.data.set_section_power.power));

      return true; // we handled this section, don't send cmd to slaves
    }
  }
  case LiketrainCommandType::SetSwitchState:
  {
    for (Switch *sw : switches)
    {
      if (!sw->matches_id(cmd.data.set_switch_state.switch_id))
        continue;

      // we found the switch
      sw->set_state(cmd.data.set_switch_state.state);

      events.enqueue(
          LiketrainEvent::switch_state_change(
              cmd.data.set_switch_state.switch_id,
              cmd.data.set_switch_state.state));

      return true; // we handled this switch, don't send to slaves
    }
  }
  case LiketrainCommandType::ResetAll:
  {
    for (Section *section : sections)
    {
      section->reset();
    }

    for (Switch *sw : switches)
    {
      sw->reset();
    }

    // don't return true, so the ResetAll command will be relayed to the slaves, which will cause them to reset as well
  }
  }

  return false;
}