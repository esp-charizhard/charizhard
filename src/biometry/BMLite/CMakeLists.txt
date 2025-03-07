idf_component_register(
    SRCS "src/bmlite_if.c"
         "src/esp_hal.c"
         "src/fpc_crc.c"
         "src/hcp_tiny.c"
         "src/platform.c"
    INCLUDE_DIRS "include"
    PRIV_INCLUDE_DIRS "src"
    REQUIRES driver freertos esp_timer)