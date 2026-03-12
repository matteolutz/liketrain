#ifndef QUEUE_H
#define QUEUE_H

#include <Arduino.h>

template <typename T>
class Queue
{
private:
    T *buffer;
    size_t buffer_size;

    bool should_free_buffer = false;

    size_t head = 0;
    size_t tail = 0;
    size_t count = 0;

public:
    /// @brief  Creates a queue with the given buffer and size. The buffer must be large enough to hold the specified number of elements.
    /// @param buffer
    /// @param buffer_size
    Queue(T *buffer, size_t buffer_size) : buffer(buffer), buffer_size(buffer_size) {}

    /// @brief Creates a queue with a new heap allocated buffer of the given size.
    /// @param buffer_size
    Queue(size_t buffer_size) : buffer(new T[buffer_size]), buffer_size(buffer_size), should_free_buffer(true) {}

    ~Queue()
    {
        if (should_free_buffer)
            delete[] buffer;
    }

    bool enqueue(T byte)
    {
        if (is_full())
            return false;

        buffer[tail] = byte;
        tail = (tail + 1) % buffer_size;
        count++;

        return true;
    }

    bool dequeue(T &byte)
    {
        if (is_empty())
            return false;

        byte = buffer[head];
        head = (head + 1) % buffer_size;
        count--;

        return true;
    }

    bool peek(T &byte) const
    {
        if (is_empty())
            return false;

        byte = buffer[head];
        return true;
    }

    bool peek_at(size_t index, T &byte) const
    {
        if (index >= count)
            return false;

        size_t actual_index = (head + index) % buffer_size;
        byte = buffer[actual_index];
        return true;
    }

    bool drain_front(size_t n)
    {
        if (n > count)
            return false;

        head = (head + n) % buffer_size;
        count -= n;

        return true;
    }

    bool extend_from(const T *data, size_t length)
    {
        if (available() < length)
            return false;

        for (size_t i = 0; i < length; i++)
        {
            if (!enqueue(data[i]))
                return false;
        }

        return true;
    }

    bool drain_into(T *data, size_t length)
    {
        if (size() < length)
            return false;

        for (size_t i = 0; i < length; i++)
        {
            if (!dequeue(data[i]))
                return false;
        }

        return true;
    }

    void clear()
    {
        head = 0;
        tail = 0;
        count = 0;
    }

    bool is_empty() const { return count == 0; }
    bool is_full() const { return count == buffer_size; }

    size_t size() const { return count; }
    size_t available() const { return buffer_size - count; }
};

#endif /// QUEUE_H