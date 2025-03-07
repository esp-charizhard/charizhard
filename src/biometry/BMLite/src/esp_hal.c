#include <driver/spi_master.h>
#include <driver/gpio.h>
#include <esp_timer.h>
#include <freertos/FreeRTOS.h>
#include <freertos/task.h>

#include "bmlite_hal.h"
#include "console_params.h"
#include "fpc_bep_types.h"
#include "platform.h"

#define BM_LITE_SPI_HOST    SPI2_HOST

// PINOUT
#define BM_LITE_CS_N_PIN GPIO_NUM_2 //io
#define BM_LITE_MISO_PIN GPIO_NUM_35 //i
#define BM_LITE_RST_PIN   GPIO_NUM_4 //io
#define BM_LITE_MOSI_PIN GPIO_NUM_12 //io 
#define BM_LITE_IRQ_PIN   GPIO_NUM_14 //io
#define BM_LITE_SPI_CLK_PIN GPIO_NUM_15 //io

static spi_device_handle_t spi_handle;

fpc_bep_result_t hal_board_init(void *params)
{
    console_initparams_t *p = (console_initparams_t *)params;

    spi_bus_config_t buscfg = {
        .miso_io_num = BM_LITE_MISO_PIN,
        .mosi_io_num = BM_LITE_MOSI_PIN,
        .sclk_io_num = BM_LITE_SPI_CLK_PIN,
        .quadwp_io_num = -1,
        .quadhd_io_num = -1,
        .max_transfer_sz = 1024,
    };

    spi_device_interface_config_t devcfg = {
        .mode = 0,
        .clock_speed_hz = p->baudrate,
        .spics_io_num = BM_LITE_CS_N_PIN,
        .queue_size = 1,
    };

    esp_err_t ret = spi_bus_initialize(BM_LITE_SPI_HOST, &buscfg, SPI_DMA_CH_AUTO);
    if (ret != ESP_OK) {
        return FPC_BEP_RESULT_INTERNAL_ERROR;
    }

    ret = spi_bus_add_device(BM_LITE_SPI_HOST, &devcfg, &spi_handle);
    if (ret != ESP_OK) {
        return FPC_BEP_RESULT_INTERNAL_ERROR;
    }

    // Init Reset Pin
    gpio_config_t io_conf = {
        .pin_bit_mask = (1ULL << BM_LITE_RST_PIN),
        .mode = GPIO_MODE_OUTPUT,
        .pull_up_en = GPIO_PULLUP_DISABLE,
        .pull_down_en = GPIO_PULLDOWN_DISABLE,
        .intr_type = GPIO_INTR_DISABLE,
    };
    gpio_config(&io_conf);

    // Init READY Pin (IRQ)
    io_conf.pin_bit_mask = (1ULL << BM_LITE_IRQ_PIN);
    io_conf.mode = GPIO_MODE_INPUT;
    gpio_config(&io_conf);

    p->hcp_comm->read = platform_bmlite_spi_receive;
    p->hcp_comm->write = platform_bmlite_spi_send;
    p->hcp_comm->phy_rx_timeout = p->timeout * 1000;

    return FPC_BEP_RESULT_OK;
}

void hal_bmlite_reset(bool state)
{
    gpio_set_level(BM_LITE_RST_PIN, state ? 0 : 1);  // Active Low
}

bool hal_bmlite_get_status(void)
{
    return gpio_get_level(BM_LITE_IRQ_PIN) == 1;  // Active High
}

fpc_bep_result_t hal_bmlite_spi_write_read(uint8_t *write, uint8_t *read, size_t size, bool leave_cs_asserted)
{
    if (size == 0) {
        return FPC_BEP_RESULT_OK;
    }

    spi_transaction_t t = {
        .length = size * 8,       // length in bits
        .tx_buffer = write,
        .rx_buffer = read,
        .flags = leave_cs_asserted ? SPI_TRANS_CS_KEEP_ACTIVE : 0,
    };

    esp_err_t ret = spi_device_transmit(spi_handle, &t);
    if (ret != ESP_OK) {
        return FPC_BEP_RESULT_IO_ERROR;
    }
    return FPC_BEP_RESULT_OK;
}


// --- Timing functions ---

void hal_timebase_init(void)
{
    // ESP32's system timer is always running, no init needed
}

hal_tick_t hal_timebase_get_tick(void)
{
    return esp_timer_get_time() / 1000;  // microseconds to milliseconds
}

void hal_timebase_busy_wait(uint32_t ms)
{
    vTaskDelay(pdMS_TO_TICKS(ms));
}

// Leave these empty if you aren't using UART
size_t hal_bmlite_uart_write(const uint8_t *data, size_t size)
{
    return 0;
}

size_t hal_bmlite_uart_read(uint8_t *buff, size_t size)
{
    return 0;
}
