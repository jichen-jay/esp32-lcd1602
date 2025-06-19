#![no_std]
#![no_main]

use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::Io,
    i2c::{I2C, Error as I2cError},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
    Blocking,
};
use esp_println::println;
use esp_backtrace as _;
use esp_hal::entry;

// LCD Constants
const LCD_ADDRESS: u8 = 0x27;
const LCD_BACKLIGHT: u8 = 0x08;
const LCD_NOBACKLIGHT: u8 = 0x00;
const EN: u8 = 0x04;
const RS: u8 = 0x01;
const LCD_CLEARDISPLAY: u8 = 0x01;
const LCD_RETURNHOME: u8 = 0x02;
const LCD_ENTRYMODESET: u8 = 0x04;
const LCD_DISPLAYCONTROL: u8 = 0x08;
const LCD_CURSORSHIFT: u8 = 0x10;
const LCD_FUNCTIONSET: u8 = 0x20;
const LCD_SETCGRAMADDR: u8 = 0x40;
const LCD_SETDDRAMADDR: u8 = 0x80;
const LCD_ENTRYLEFT: u8 = 0x02;
const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;
const LCD_DISPLAYON: u8 = 0x04;
const LCD_CURSOROFF: u8 = 0x00;
const LCD_BLINKOFF: u8 = 0x00;
const LCD_4BITMODE: u8 = 0x00;
const LCD_2LINE: u8 = 0x08;
const LCD_1LINE: u8 = 0x00;
const LCD_5X8DOTS: u8 = 0x00;

struct SimpleLcd<'a, T> {
    i2c: I2C<'a, T, Blocking>,
    backlight: u8,
    display_control: u8,
    display_mode: u8,
    delay: Delay,
}

impl<'a, T> SimpleLcd<'a, T>
where
    T: esp_hal::i2c::Instance,
{
    fn new(i2c: I2C<'a, T, Blocking>, delay: Delay) -> Self {
        Self {
            i2c,
            backlight: LCD_NOBACKLIGHT,
            display_control: LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF,
            display_mode: LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT,
            delay,
        }
    }

    fn init(&mut self) -> Result<(), &'static str> {
        // Wait for LCD to power up
        self.delay.delay_millis(50u32);

        // Initialize in 4-bit mode
        self.expander_write(self.backlight)?;
        self.delay.delay_millis(1000u32);

        // Send initialization sequence
        for _ in 0..3 {
            self.write4bits((0x03 << 4) | self.backlight)?;
            self.delay.delay_micros(4500u32);
        }

        // Set to 4-bit mode
        self.write4bits((0x02 << 4) | self.backlight)?;

        // Function set: 4-bit, 2 lines, 5x8 dots
        let display_function = LCD_4BITMODE | LCD_2LINE | LCD_5X8DOTS;
        self.send(LCD_FUNCTIONSET | display_function, 0)?;

        // Display on
        self.display_control = LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF;
        self.send(LCD_DISPLAYCONTROL | self.display_control, 0)?;

        // Clear display
        self.clear()?;

        // Entry mode set
        self.display_mode = LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT;
        self.send(LCD_ENTRYMODESET | self.display_mode, 0)?;

        // Return home
        self.send(LCD_RETURNHOME, 0)?;
        self.delay.delay_micros(2000u32);

        Ok(())
    }

    fn backlight_on(&mut self) -> Result<(), &'static str> {
        self.backlight = LCD_BACKLIGHT;
        self.expander_write(self.backlight)?;
        Ok(())
    }

    fn clear(&mut self) -> Result<(), &'static str> {
        self.send(LCD_CLEARDISPLAY, 0)?;
        self.delay.delay_micros(2000u32);
        Ok(())
    }

    fn set_cursor(&mut self, col: u8, row: u8) -> Result<(), &'static str> {
        let row_offsets = [0x00, 0x40];
        if row >= 2 {
            return Err("Row out of bounds");
        }
        let cmd = LCD_SETDDRAMADDR | (col + row_offsets[row as usize]);
        self.send(cmd, 0)?;
        Ok(())
    }

    fn print_char(&mut self, ch: char) -> Result<(), &'static str> {
        self.send(ch as u8, RS)?;
        Ok(())
    }

    fn print_str(&mut self, s: &str) -> Result<(), &'static str> {
        for ch in s.chars() {
            self.print_char(ch)?;
        }
        Ok(())
    }

    fn expander_write(&mut self, data: u8) -> Result<(), &'static str> {
        let bytes = [data];
        self.i2c.write(LCD_ADDRESS, &bytes).map_err(|_| "I2C write failed")?;
        Ok(())
    }

    fn pulse_enable(&mut self, data: u8) -> Result<(), &'static str> {
        let pulse = (data | EN) | self.backlight;
        self.expander_write(pulse)?;
        self.delay.delay_micros(1u32);

        let pulse = (data & !EN) | self.backlight;
        self.expander_write(pulse)?;
        self.delay.delay_micros(50u32);
        Ok(())
    }

    fn write4bits(&mut self, data: u8) -> Result<(), &'static str> {
        self.expander_write(data)?;
        self.pulse_enable(data)?;
        Ok(())
    }

    fn send(&mut self, value: u8, mode: u8) -> Result<(), &'static str> {
        let high_nibble = value & 0xf0;
        let low_nibble = (value << 4) & 0xf0;

        let high_cmd = (high_nibble | mode) | self.backlight;
        self.write4bits(high_cmd)?;

        let low_cmd = (low_nibble | mode) | self.backlight;
        self.write4bits(low_cmd)?;
        Ok(())
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

    // Configure I2C pins
    // GPIO21 for SDA, GPIO22 for SCL
    let sda = io.pins.gpio21;
    let scl = io.pins.gpio22;

    // Initialize I2C
    let i2c = I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        100u32.kHz(),
        &clocks,
    );

    println!("I2C initialized!");

    // Create LCD instance
    let mut lcd = SimpleLcd::new(i2c, delay.clone());

    // Initialize the LCD
    match lcd.init() {
        Ok(_) => println!("LCD initialized successfully!"),
        Err(e) => println!("Failed to initialize LCD: {}", e),
    }

    // Turn on backlight
    let _ = lcd.backlight_on();

    // Clear the display
    let _ = lcd.clear();

    // Set cursor to beginning of first line
    let _ = lcd.set_cursor(0, 0);

    // Print "Happy Bday!"
    match lcd.print_str("Happy Bday!") {
        Ok(_) => println!("Message displayed successfully!"),
        Err(e) => println!("Failed to display message: {}", e),
    }

    // Add a fun message on the second line
    let _ = lcd.set_cursor(0, 1);
    let _ = lcd.print_str("Enjoy your day!");

    // Main loop
    loop {
        // Just keep the program running
        delay.delay_millis(1000u32);
    }
}
