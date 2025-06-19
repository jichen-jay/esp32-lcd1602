# LCD1602 to ESP32-C6 Wiring Guide

## Pin Connections (4-bit mode)

### LCD1602 Pin Layout:
```
LCD1602 (16 pins):
1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16
VSS VDD V0 RS RW E  D0 D1 D2 D3 D4 D5 D6 D7 A  K
```

### Wiring Table:

| LCD Pin | LCD Function | ESP32-C6 Pin | Connection Notes |
|---------|--------------|--------------|------------------|
| 1 (VSS) | Ground | GND | Power ground |
| 2 (VDD) | Power +5V | 5V | Power supply (or 3.3V if LCD supports it) |
| 3 (V0) | Contrast | GND via 10kΩ pot | Connect middle pin of potentiometer |
| 4 (RS) | Register Select | GPIO0 | Command/Data select |
| 5 (RW) | Read/Write | GND | Ground (we only write) |
| 6 (E) | Enable | GPIO1 | Enable signal |
| 7 (D0) | Data 0 | Not connected | Not used in 4-bit mode |
| 8 (D1) | Data 1 | Not connected | Not used in 4-bit mode |
| 9 (D2) | Data 2 | Not connected | Not used in 4-bit mode |
| 10 (D3) | Data 3 | Not connected | Not used in 4-bit mode |
| 11 (D4) | Data 4 | GPIO2 | Data bit 4 |
| 12 (D5) | Data 5 | GPIO3 | Data bit 5 |
| 13 (D6) | Data 6 | GPIO4 | Data bit 6 |
| 14 (D7) | Data 7 | GPIO5 | Data bit 7 |
| 15 (A) | Backlight + | 5V or 3.3V | Via 220Ω resistor |
| 16 (K) | Backlight - | GND | Backlight ground |

## Required Components:

1. **10kΩ Potentiometer** - For contrast adjustment
   - One end to 5V/3.3V
   - Other end to GND
   - Middle pin (wiper) to LCD pin 3 (V0)

2. **220Ω Resistor** - For backlight current limiting
   - Between power supply and LCD pin 15 (A)

## Wiring Diagram (ASCII):

```
ESP32-C6          LCD1602
--------          -------
GND    ---------> 1 (VSS)
5V     ---------> 2 (VDD)
               -> 3 (V0) <-- 10kΩ pot wiper
GPIO0  ---------> 4 (RS)
GND    ---------> 5 (RW)
GPIO1  ---------> 6 (E)
                  7-10 (D0-D3) [Not connected]
GPIO2  ---------> 11 (D4)
GPIO3  ---------> 12 (D5)
GPIO4  ---------> 13 (D6)
GPIO5  ---------> 14 (D7)
5V/3.3V --[220Ω]-> 15 (A)
GND    ---------> 16 (K)
```

## Alternative Pin Mapping (if GPIOs 0-5 are not available):

You can change the GPIO pins in the code. Here's an alternative mapping using higher GPIO numbers:

| Function | Alternative GPIO |
|----------|------------------|
| RS | GPIO21 |
| Enable | GPIO22 |
| D4 | GPIO23 |
| D5 | GPIO8 |
| D6 | GPIO9 |
| D7 | GPIO10 |

Just update the pin assignments in the code accordingly.

## Power Considerations:

- Most LCD1602 displays work with 5V power but have 3.3V compatible logic levels
- If your LCD requires 5V logic levels, you may need level shifters
- The ESP32-C6 GPIO pins are 3.3V - most LCD1602 modules accept this
- Current consumption: ~20mA for LCD + ~20mA for backlight

## Troubleshooting:

1. **No display**: Check power connections and contrast adjustment
2. **Blocks/squares only**: Usually indicates initialization failure or wrong wiring
3. **Dim display**: Adjust the contrast potentiometer
4. **No backlight**: Check resistor and backlight connections
5. **Garbled text**: Check data pin connections (D4-D7)
