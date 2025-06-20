#![no_std]
#![no_main]

use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{Io, Level, Output},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};
use esp_println::println;
use esp_backtrace as _;
use esp_hal::entry;

// LCD Commands
const LCD_CLEARDISPLAY: u8 = 0x01;
const LCD_RETURNHOME: u8 = 0x02;
const LCD_ENTRYMODESET: u8 = 0x04;
const LCD_DISPLAYCONTROL: u8 = 0x08;
const LCD_CURSORSHIFT: u8 = 0x10;
const LCD_FUNCTIONSET: u8 = 0x20;
const LCD_SETCGRAMADDR: u8 = 0x40;
const LCD_SETDDRAMADDR: u8 = 0x80;

// Entry mode
const LCD_ENTRYLEFT: u8 = 0x02;
const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

// Display control
const LCD_DISPLAYON: u8 = 0x04;
const LCD_CURSOROFF: u8 = 0x00;
const LCD_BLINKOFF: u8 = 0x00;

// Function set
const LCD_8BITMODE: u8 = 0x10;
const LCD_4BITMODE: u8 = 0x00;
const LCD_2LINE: u8 = 0x08;
const LCD_1LINE: u8 = 0x00;
const LCD_5X8DOTS: u8 = 0x00;

struct Lcd1602<'a> {
    rs: Output<'a>,
    enable: Output<'a>,
    d4: Output<'a>,
    d5: Output<'a>,
    d6: Output<'a>,
    d7: Output<'a>,
    delay: Delay,
    display_control: u8,
    display_mode: u8,
}

impl<'a> Lcd1602<'a> {
    fn new(
        rs: Output<'a>,
        enable: Output<'a>,
        d4: Output<'a>,
        d5: Output<'a>,
        d6: Output<'a>,
        d7: Output<'a>,
        delay: Delay,
    ) -> Self {
        Self {
            rs,
            enable,
            d4,
            d5,
            d6,
            d7,
            delay,
            display_control: LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF,
            display_mode: LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT,
        }
    }

    fn init(&mut self) {
        // Wait for power stabilization
        self.delay.delay_millis(50);

        // Set all pins low
        self.rs.set_low();
        self.enable.set_low();

        // Initialize in 4-bit mode
        // Send 0x03 three times
        for _ in 0..3 {
            self.write4bits(0x03);
            self.delay.delay_micros(4500);
        }

        // Set to 4-bit mode
        self.write4bits(0x02);

        // Function set: 4-bit, 2 lines, 5x8 dots
        self.command(LCD_FUNCTIONSET | LCD_4BITMODE | LCD_2LINE | LCD_5X8DOTS);

        // Display on/off control
        self.command(LCD_DISPLAYCONTROL | self.display_control);

        // Clear display
        self.clear();

        // Entry mode set
        self.command(LCD_ENTRYMODESET | self.display_mode);
    }

    fn clear(&mut self) {
        self.command(LCD_CLEARDISPLAY);
        self.delay.delay_micros(2000);
    }

    fn home(&mut self) {
        self.command(LCD_RETURNHOME);
        self.delay.delay_micros(2000);
    }

    fn set_cursor(&mut self, col: u8, row: u8) {
        let row_offsets = [0x00, 0x40, 0x14, 0x54];
        if row < 2 {
            self.command(LCD_SETDDRAMADDR | (col + row_offsets[row as usize]));
        }
    }

    fn print(&mut self, text: &str) {
        for ch in text.chars() {
            self.write(ch as u8);
        }
    }

    fn command(&mut self, value: u8) {
        self.send(value, false);
    }

    fn write(&mut self, value: u8) {
        self.send(value, true);
    }

    fn send(&mut self, value: u8, is_data: bool) {
        if is_data {
            self.rs.set_high();
        } else {
            self.rs.set_low();
        }

        // Send high nibble
        self.write4bits((value >> 4) & 0x0F);
        // Send low nibble
        self.write4bits(value & 0x0F);
    }

    fn write4bits(&mut self, value: u8) {
        // Set data pins
        if value & 0x01 != 0 { self.d4.set_high(); } else { self.d4.set_low(); }
        if value & 0x02 != 0 { self.d5.set_high(); } else { self.d5.set_low(); }
        if value & 0x04 != 0 { self.d6.set_high(); } else { self.d6.set_low(); }
        if value & 0x08 != 0 { self.d7.set_high(); } else { self.d7.set_low(); }

        // Pulse enable
        self.enable.set_low();
        self.delay.delay_micros(1);
        self.enable.set_high();
        self.delay.delay_micros(1);
        self.enable.set_low();
        self.delay.delay_micros(100);
    }
}

#[entry]
fn main() -> ! {
    // Take peripherals
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Initialize IO
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Set up delay
    let delay = Delay::new(&clocks);

    println!("Initializing LCD1602...");

    // Configure LCD pins (using 4-bit mode to save pins)
    // Control pins
    let rs = Output::new(io.pins.gpio0, Level::Low);     // Register Select
    let enable = Output::new(io.pins.gpio1, Level::Low); // Enable

    // Data pins (only using D4-D7 for 4-bit mode)
    let d4 = Output::new(io.pins.gpio2, Level::Low);
    let d5 = Output::new(io.pins.gpio3, Level::Low);
    let d6 = Output::new(io.pins.gpio4, Level::Low);
    let d7 = Output::new(io.pins.gpio5, Level::Low);

    // Create LCD instance
    let mut lcd = Lcd1602::new(rs, enable, d4, d5, d6, d7, delay.clone());

    // Initialize LCD
    lcd.init();
    println!("LCD initialized!");

    // Display message
    lcd.clear();
    lcd.set_cursor(0, 0);
    lcd.print("Happy Bday!");

    lcd.set_cursor(0, 1);
    lcd.print("Enjoy your day!");

    println!("Message displayed!");

    // Create a blinking cursor effect
    let mut blink = false;

    // Main loop
    loop {
        delay.delay_millis(500);

        // Optional: Add a blinking heart or star
        lcd.set_cursor(15, 0);
        if blink {
            lcd.print("*");
        } else {
            lcd.print(" ");
        }
        blink = !blink;
    }
}
