use at32f4xx_hal::flash::FlashExt;
use embassy_time::Timer;
use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash};
use sequential_storage::{
    cache::{KeyCacheImpl, NoCache},
    map::{MapConfig, MapStorage},
};

use crate::cfg::{HeadlightMode, Odometer, SpeedLimit, SpeedMode, Storable, UnlockCode};

unsafe extern "C" {
    static __config_start: u32;
    static __config_end: u32;
}

fn config_start() -> usize {
    unsafe { &__config_start as *const u32 as usize }
}

fn config_end() -> usize {
    unsafe { &__config_end as *const u32 as usize }
}

#[embassy_executor::task]
pub async fn worker(flash: at32f4xx_hal::pac::FLASH) {
    worker_(flash).await;
}

pub async fn worker_(flash: at32f4xx_hal::pac::FLASH) {
    defmt::debug!(
        "FLASH INFO: start: {:x}, end: {:x}, len: {:x}",
        config_start(),
        config_end(),
        config_end() - config_start()
    );

    let mut buffer = [0u8; 32];

    let mut map_storage =
        MapStorage::<u8, _, _>::new(MyFlash(flash), MapConfig::new(0..8192), NoCache);

    init_stored::<SpeedLimit, _, _>(&mut map_storage, &mut buffer);
    init_stored::<HeadlightMode, _, _>(&mut map_storage, &mut buffer);
    init_stored::<SpeedMode, _, _>(&mut map_storage, &mut buffer);
    init_stored::<UnlockCode, _, _>(&mut map_storage, &mut buffer);
    init_stored::<Odometer, _, _>(&mut map_storage, &mut buffer);

    loop {
        Timer::after_secs(10).await;

        write_stored_if_changed::<SpeedLimit, _, _>(&mut map_storage, &mut buffer);
        write_stored_if_changed::<HeadlightMode, _, _>(&mut map_storage, &mut buffer);
        write_stored_if_changed::<SpeedMode, _, _>(&mut map_storage, &mut buffer);
        write_stored_if_changed::<Odometer, _, _>(&mut map_storage, &mut buffer);
    }
}

fn init_stored<T: Storable, S: NorFlash, C: KeyCacheImpl<u8>>(
    map_storage: &mut MapStorage<u8, S, C>,
    buf: &mut [u8],
) {
    match embassy_futures::block_on(map_storage.fetch_item::<T>(buf, &T::ID)) {
        Ok(Some(v)) => {
            T::update_stored(v);
            let _ = T::take_if_changed_and_timedout();
        }
        r => {
            if r.is_err() {
                defmt::warn!("Failed to fetch entry for id {}, loading default", T::ID);
            } else {
                defmt::debug!("No stored entry found for id {}, loading default", T::ID);
            }
            T::update_stored(T::default());
            let _ = T::mark_unchanged();
        }
    }
}

fn write_stored_if_changed<T: Storable, S: NorFlash, C: KeyCacheImpl<u8>>(
    map_storage: &mut MapStorage<u8, S, C>,
    buf: &mut [u8],
) {
    if let Some(v) = T::take_if_changed_and_timedout() {
        if let Err(_) = embassy_futures::block_on(map_storage.store_item(buf, &T::ID, &v)) {
            defmt::warn!("Failed to write changed item for id {}", T::ID);
        } else {
            defmt::debug!("Updated entry for id {}", T::ID);
        }
    }
}

struct MyFlash(at32f4xx_hal::pac::FLASH);

impl embedded_storage_async::nor_flash::ErrorType for MyFlash {
    type Error = at32f4xx_hal::flash::Error;
}

impl ReadNorFlash for MyFlash {
    const READ_SIZE: usize = 1;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        let config_offset = config_start() - self.0.address();
        let offset = offset.saturating_add(config_offset);

        defmt::trace!(
            "(app) Reading flash at {:x} for {} bytes",
            offset,
            bytes.len()
        );

        // dunno if we need to do anything better here to prevent fuckery
        for (src, dst) in self.0.read()[offset..].iter().zip(bytes.iter_mut()) {
            *dst = *src;
        }

        Ok(())
    }

    fn capacity(&self) -> usize {
        config_end() - config_start()
    }
}

impl NorFlash for MyFlash {
    const WRITE_SIZE: usize = 1;
    const ERASE_SIZE: usize = 2048;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let config_offset = config_start() - self.0.address();
        let from = from.saturating_add(config_offset as u32);
        let to = to.saturating_add(config_offset as u32);

        let mut unlocked = self.0.unlocked();

        defmt::trace!("(app) Erasing flash from {:x} to {:x}", from, to);

        embedded_storage::nor_flash::NorFlash::erase(&mut unlocked, from, to)?;

        Ok(())
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let config_offset = config_start() - self.0.address();
        let offset = offset.saturating_add(config_offset as u32);

        let mut unlocked = self.0.unlocked();

        defmt::trace!(
            "(app) Writing flash at {:x} with {} bytes",
            offset,
            bytes.len()
        );

        embedded_storage::nor_flash::NorFlash::write(&mut unlocked, offset, bytes)?;

        Ok(())
    }
}
