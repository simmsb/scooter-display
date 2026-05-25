MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  /* TODO Adjust these memory regions to match your device memory layout */
  FLASH : ORIGIN = 0x8000000, LENGTH = 240K /* AT32F415RCT6 has 256K, but be conservative */
  /* FLASH : ORIGIN = 0x8008000, LENGTH = 64K /\* AT32F415RCT6 has 256K, but be conservative *\/ */
  RAM : ORIGIN = 0x20000000, LENGTH = 32K
}
