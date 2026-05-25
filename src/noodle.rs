use cfg_noodle::{
    StorageList, StorageListNode,
    minicbor::{self, CborLen, Decode, Encode},
    mutex::raw_impls::cs::CriticalSectionRawMutex,
    sequential_storage::cache::{NoCache, PagePointerCache},
};
use static_cell::ConstStaticCell;

const SECTOR_SIZE: u32 = 4096;
const PAGE_SIZE: u32 = 256;

// give ourselves 128KB of the 8MB flash.
const TOTAL_SIZE: u32 = 128 * 1024;
const PAGE_COUNT: u32 = TOTAL_SIZE / SECTOR_SIZE;

// start of flash contains some images used by original firmware, but it is a
// 128mib flash with everything after 0x400000 free.
const BASE_ADDR: u32 = 0x500000;

pub static LIST: StorageList<CriticalSectionRawMutex, 3> = StorageList::new();

pub static BUF: ConstStaticCell<[u8; 64]> = ConstStaticCell::new([0u8; 64]);

#[embassy_executor::task]
pub async fn worker(spi: at32f4xx_hal::spi::Spi<at32f4xx_hal::spi::mode::Master>) {
    worker_(spi).await;
}

pub async fn worker_(spi: at32f4xx_hal::spi::Spi<at32f4xx_hal::spi::mode::Master>) {
    let mut flash = w25::W25::<w25::Q, _, _, _>::new(
        spi,
        dummy_pin::DummyPin::new_high(),
        dummy_pin::DummyPin::new_high(),
        PAGE_SIZE * 65536,
    )
    .unwrap();

    flash.reset().await.unwrap();

    let flash_id = flash.device_id().await.unwrap();
    defmt::info!("Flash device id: {}", flash_id);
    let mut first_10_bytes = [0u8; 10];
    flash.read(0, &mut first_10_bytes).await.unwrap();
    defmt::info!("Flash device bytes: {}", first_10_bytes);

    let buf: &mut [u8] = BUF.take().as_mut_slice();

    let wrapped_flash = cfg_noodle::flash::Flash::new(
        flash,
        BASE_ADDR..(BASE_ADDR + TOTAL_SIZE as u32),
        NoCache::new(),
    );

    cfg_noodle::worker_task::default_worker_task(
        &LIST,
        wrapped_flash,
        core::future::pending::<core::convert::Infallible>(),
        buf,
    )
    .await;
}
