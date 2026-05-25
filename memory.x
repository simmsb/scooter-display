MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  FLASH  : ORIGIN = 0x8000000, LENGTH = 220K /* AT32F415RCT6 has 256K, but be conservative */
  /* 4k from end of flash */
  CONFIG : ORIGIN = ORIGIN(FLASH) + LENGTH(FLASH), LENGTH = 16k
  /* FLASH : ORIGIN = 0x8008000, LENGTH = 64K /\* AT32F415RCT6 has 256K, but be conservative *\/ */
  RAM : ORIGIN = 0x20000000, LENGTH = 32K
}

__config_start = ORIGIN(CONFIG); /* - ORIGIN(FLASH); */
__config_end = ORIGIN(CONFIG) + LENGTH(CONFIG) /* - ORIGIN(FLASH) */;

SECTIONS {
  .config (NOLOAD) : ALIGN(4)
  {
    . = ALIGN(4);
    *(.config .config.*);
    . = ALIGN(4);
  } > CONFIG
}
