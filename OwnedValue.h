/**
 * OwnedValue.h
 *
 * A simple ownership model for C++ with runtime checks
 * 
 * Simple usage example:
 * ```cpp
 * #include <OwnedValue.h>
 *
 * void foo() {
 *     OwnedValue<int> x{ 0 };
 *
 *     {
 *         OwnedValue<int>::Ref ref{ x };
 *         int y = ref.read();
 *
 *         OwnedValue<int>::RefMut ref_mut{ x };
 *         ref_mut.write(1); // Crash: Ownership of `ref` not respected
 *     }
 * }
 * ```
 *
 */
#pragma once

#include <iostream>
#include <map>
#include <memory>
#include <vector>

// Enable for more aggressive borrow checking (less tolerant)
// #define OWNED_VALUE_BORROW_CHECK_AGGRESSIVE

template <typename T>
class OwnedValue {
private:
    class ResolvableRef {
    public:
        T read() { return get(); }
        void write(T value);

        T& get();
        const T& get() const;

        ResolvableRef() = delete;
        ResolvableRef(void* ref_owner, const OwnedValue* value, bool writeable);

        ~ResolvableRef();

        const void* ref_owner() const { return ref_owner_; }

        const OwnedValue* value() const { return value_; }
        void set_value(const OwnedValue* new_value) { value_ = new_value; }

    private:
        void resolve();

        void* ref_owner_ { nullptr };
        const OwnedValue* value_ { nullptr };
        bool writeable_ { false };
        mutable bool is_resolved_ { false };
    };

    struct Owners {
        std::vector<ResolvableRef*> readers {};
        ResolvableRef* writer { nullptr };

        Owners() {}

        void panic_writing_to_concurrent_read(const void* owner) const;
        void panic_reading_to_concurrent_write(const void* owner) const;
    };

    // Tracing of owners for each currently visible OwnedValue.
    // We do this globally in order to keep sizeof(OwnedValue<T>) == sizeof(T)
    static std::map<const OwnedValue*, Owners>* owners() {
        thread_local static std::unique_ptr<std::map<const OwnedValue*, Owners>> owners_ {
            nullptr
        };

        if (!owners_) {
            owners_.reset(new std::map<const OwnedValue*, Owners>());
        }

        return owners_.get();
    }

public:
    class Ref {
    public:
        Ref() = delete;
        Ref(const OwnedValue& value)
            : ref_(__builtin_return_address(0), &value, false) {}

        Ref(const Ref& other)
            : ref_(__builtin_return_address(0), other.value(), false) {}

        // A copy on move is fine here since it is a readonly ref anyway thus ordering of
        // destructing the old ref / constructing the new ref is not important
        Ref(Ref&& other)
            : ref_(__builtin_return_address(0), other.value(), false) {}

        T read() const { return ref_.read(); }

        const T& get() const { return ref_.get(); }

    private:
        mutable ResolvableRef ref_;
    };

    class RefMut {
    public:
        RefMut() = delete;
        RefMut(OwnedValue& value)
            : ref_(__builtin_return_address(0), &value, true) {}

        RefMut(const RefMut&) = delete;
        RefMut(RefMut&&) = delete;

        T read() const { return ref_.read(); }
        void write(T value) { ref_.write(value); }

        T& get() { return ref_.get(); }
        const T& get() const { return ref_.get(); }

    private:
        mutable ResolvableRef ref_;
    };

    explicit OwnedValue(T value)
        : value_ { value } {}

    ~OwnedValue() {
        auto it = owners()->find(this);
        if (it != owners()->end()) {
            owners()->erase(it);
        }
    }

    // This is not nescessary but it will give us a better compiler support when integrating
    OwnedValue(const OwnedValue&) = delete;

    OwnedValue(OwnedValue<T>&& other) { *this = std::move(other); }
    OwnedValue<T>& operator=(OwnedValue<T>&& rhs) {
        auto it = owners()->find(&rhs);
        if (it != owners().end()) {

            for (auto& ref : it->second.readers) {
                ref->set_value(this);
            }
            if (it->second.writer != nullptr) {
                it->second.writer->set_value(this);
            }

            owners()->emplace(this, std::move(it->second));
            owners()->erase(it);
        }

        value_ = std::move(rhs.value_);

        return *this;
    }

private:
    mutable T value_;
};

template <typename T>
OwnedValue<T>::ResolvableRef::ResolvableRef(void* ref_owner,
                                            const OwnedValue<T>* value,
                                            bool writeable)
    : ref_owner_(ref_owner)
    , value_(value)
    , writeable_(writeable) {

    if (!OwnedValue<T>::owners()->count(value_)) {
        (*OwnedValue<T>::owners())[value_];
    }

#ifdef OWNED_VALUE_BORROW_CHECK_AGGRESSIVE
    resolve();
#endif
}

template <typename T>
void OwnedValue<T>::ResolvableRef::resolve() {
    is_resolved_ = true;
    auto& owners = OwnedValue<T>::owners()->at(value_);

    if (writeable_) {
        if (!owners.readers.empty()) {
            owners.panic_writing_to_concurrent_read(ref_owner_);
            return;
        }

        if (owners.writer != nullptr) {
            // error. panic I suppose?
            std::cerr << __PRETTY_FUNCTION__
                      << ": Implementation error, reader not set but writer == " << owners.writer
                      << " ?\n";

            abort();
        }

        owners.writer = this;
    } else if (owners.writer != nullptr) {
        owners.panic_reading_to_concurrent_write(ref_owner_);
        return;
    }

    owners.readers.emplace_back(this);
}

template <typename T>
OwnedValue<T>::ResolvableRef::~ResolvableRef() {
    if (!is_resolved_) {
        return;
    }

    auto& owners = OwnedValue<T>::owners()->at(value_);

    if (writeable_) {
        owners.writer = nullptr;
    }

    for (auto it = owners.readers.begin(); it != owners.readers.end(); ++it) {
        if (*it == this) {
            owners.readers.erase(it);
            break;
        }
    }
}

template <typename T>
T& OwnedValue<T>::ResolvableRef::get() {
    if (!is_resolved_) {
        resolve();
    }

    return value_->value_;
}

template <typename T>
const T& OwnedValue<T>::ResolvableRef::get() const {
    if (!is_resolved_) {
        resolve();
    }

    return value_->value_;
}

template <typename T>
void OwnedValue<T>::ResolvableRef::write(T value) {
    if (!is_resolved_) {
        resolve();
    }

    value_->value_ = value;
}

template <typename T>
void OwnedValue<T>::Owners::panic_reading_to_concurrent_write(const void* owner) const {
    std::cerr << __PRETTY_FUNCTION__ << ": The owner " << owner
              << " was reading a concurrent write by " << writer->ref_owner() << "!\n";
    abort();
}
template <typename T>
void OwnedValue<T>::Owners::panic_writing_to_concurrent_read(const void* owner) const {
    std::cerr << __PRETTY_FUNCTION__ << ": The owner " << owner
              << " was writing to a value read by " << readers.size() << " concurrent reader(s):\n";
    for (const auto& r : readers) {
        std::cerr << "  " << r->ref_owner() << "\n";
    }
    abort();
}
