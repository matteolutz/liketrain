#include "lcd.h"

void TextLayer::update()
{
    if (!scrolling)
        return;

    unsigned long current_time_ms = millis();
    if (current_time_ms - last_scroll_time_ms >= scroll_interval_ms)
    {
        scroll_offset = (scroll_offset + 1) % (text.length() + scroll_spacing);
        last_scroll_time_ms = current_time_ms;
    }
}

void TextLayer::draw(Lcd *lcd)
{
    lcd->getLcd().setCursor(line.start_col, line.row);

    uint8_t end_col = line.end_col == -1 ? lcd_range.end.col - 1 : line.end_col;
    uint8_t n_cols = end_col - line.start_col + 1;

    if (scrolling)
    {
        // print the visible portion of the padded text based on the scroll offset
        for (uint8_t i = 0; i < n_cols; i++)
        {
            char c = text[(scroll_offset + i) % text.length()];
            lcd->getLcd().write(c);
        }
    }
    else
    {
        // non-scrolling: just print the substring that fits in the layer
        lcd->getLcd().print(text.substring(0, n_cols));
    }
}

Lcd::Lcd(uint8_t lcd_addr, uint8_t lcd_cols, uint8_t lcd_rows)
    : lcd(lcd_addr, lcd_cols, lcd_rows), cols(lcd_cols), rows(lcd_rows)
{
}

Lcd::~Lcd()
{
    for (size_t i = 0; i < LCD_MAX_LAYERS; i++)
    {
        if (layers[i] != nullptr)
        {
            delete layers[i];
            layers[i] = nullptr;
        }
    }
}

void Lcd::init()
{
    lcd.init();
    lcd.backlight();
}

void Lcd::update()
{
    lcd.clear();

    // update and draw the layers
    for (size_t i = 0; i < LCD_MAX_LAYERS; i++)
    {
        if (layers[i] != nullptr)
        {
            layers[i]->update();
            layers[i]->draw(this);
        }
    }
}