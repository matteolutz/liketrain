#ifndef LCD_H
#define LCD_H

#include <Arduino.h>
#include <LiquidCrystal_I2C.h>

struct LcdCursor
{
    uint8_t col;
    uint8_t row;
};

struct LcdRange
{
    LcdCursor start;
    LcdCursor end;
};

struct LcdLine
{
    uint8_t row;
    uint8_t start_col;
    int8_t end_col; // if -1, will be set to max column in layer init
};

// forward declaration of Lcd class to avoid circular dependency with LcdLayer
class Lcd;

class LcdLayer
{
public:
    virtual ~LcdLayer() = default;

    void init(LcdRange range)
    {
        lcd_range = range;
    }

    virtual void update() = 0;
    virtual void draw(Lcd *lcd) = 0;

protected:
    LcdRange lcd_range;
};

class TextLayer : public LcdLayer
{
public:
    TextLayer(const String &text, uint8_t row, uint8_t start_col, int8_t end_col = -1, bool scrolling = false)
        : line({row, start_col, end_col}), scrolling(scrolling)
    {
        if (scrolling)
        {
            // add extra spaces to the end of the text for smooth scrolling
            this->text = text + String(' ', scroll_spacing);
        }
        else
        {
            this->text = text;
        }
    }

    void update() override;
    void draw(Lcd *lcd) override;

private:
    String text;
    LcdLine line;

    bool scrolling;
    uint8_t scroll_offset = 0;
    uint8_t scroll_spacing = 3; // number of blank spaces between scroll loops

    unsigned long scroll_interval_ms = 500;
    unsigned long last_scroll_time_ms = 0;
};

#define LCD_MAX_LAYERS 4

class Lcd
{
public:
    Lcd(uint8_t lcd_addr, uint8_t lcd_cols, uint8_t lcd_rows);
    ~Lcd();

    void init();
    void update();

    LiquidCrystal_I2C &getLcd() { return lcd; }

    template <typename T>
    LcdLayer *addLayer(T layer)
    {
        for (size_t i = 0; i < LCD_MAX_LAYERS; i++)
        {
            if (layers[i] == nullptr)
            {
                LcdLayer *new_layer = new T(layer);
                new_layer->init({.start = {0, 0},
                                 .end = {cols - (uint8_t)1, rows - (uint8_t)1}});
                layers[i] = new_layer;
                return new_layer;
            }
        }

        return nullptr;
    }

private:
    LiquidCrystal_I2C lcd;
    uint8_t cols, rows;

    LcdLayer *layers[LCD_MAX_LAYERS] = {nullptr};
};

#endif // LCD_H