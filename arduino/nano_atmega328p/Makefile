MCU ?= atmega328p
TARGET ?= blink

$(TARGET).hex: $(TARGET).elf
	avr-objcopy -O ihex -R .eeprom $< $@

$(TARGET).elf: $(TARGET).c
	avr-gcc -Os -mmcu=$(MCU) -o $@ $<

.PHONY: clean
clean:
	rm *.elf *.hex

