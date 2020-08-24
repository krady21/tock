use kernel::common::cells::TakeCell;
use kernel::debug;
use kernel::hil;
use stm32f303xc::flash::StmF303Page;
use stm32f303xc::flash::Flash;

pub struct FlashUser<'a> {
    pub driver: &'a Flash,
    pub buffer: TakeCell<'static, StmF303Page>,
}

impl<'a> FlashUser<'a> {
    pub fn new(driver: &'a Flash, buffer: &'static mut StmF303Page) -> FlashUser<'a> {
        FlashUser {
            driver,
            buffer: TakeCell::new(buffer),
        }
    }

    pub fn test_erase(&self) {
        self.driver.erase_page(127);
    }

    pub fn test_write(&self) {
        self.buffer.take().map(|buffer| {
            self.driver.write_page(127, buffer);
        });
    }

    pub fn test_read(&self) {
        self.buffer.take().map(|buffer| {
            self.driver.read_page(127, buffer);
        });
    }

    pub fn start_test_option(&self) {
        self.driver.erase_option();
    }
}

impl<'a> hil::flash::Client<Flash> for FlashUser<'a> {
    fn read_complete(&self, buffer: &'static mut StmF303Page, error: hil::flash::Error) {
        debug!("Read happened");
        // self.buffer.replace(buffer);
        panic!("First three bytes: {}, {}, {}", buffer[2045], buffer[2046], buffer[2047]);
    }
    fn write_complete(&self, buffer: &'static mut StmF303Page, error: hil::flash::Error) {
        debug!("Write happened");
        self.driver.read_page(127, buffer);
    }
    fn erase_complete(&self, error: hil::flash::Error) {
        debug!("Erase happened");
        self.buffer.take().map(|buffer| {
            self.driver.write_page(127, buffer);
        });
    }
}

