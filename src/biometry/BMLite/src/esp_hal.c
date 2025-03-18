#include <driver/spi_master.h>
#include <driver/gpio.h>
#include <esp_timer.h>
#include <esp_log.h>
#include <freertos/FreeRTOS.h>
#include <freertos/task.h>

#include "bmlite_hal.h"
#include "console_params.h"
#include "fpc_bep_types.h"
#include "platform.h"

#define TAG "esp_hal"

static spi_device_handle_t spi_handle;
static pin_config_t *pins;

fpc_bep_result_t hal_board_deinit(void *params)
{
    esp_err_t ret;

    console_initparams_t *p = (console_initparams_t *)params;

    if (spi_handle) {
        ret = spi_bus_remove_device(spi_handle);
        if (ret != ESP_OK) {
            ESP_LOGE(TAG, "Failed to remove SPI device: %d", ret);
            return FPC_BEP_RESULT_INTERNAL_ERROR;
        }
        spi_handle = NULL;
    }

    if (p && p->pins) {
        ret = spi_bus_free(p->pins->spi_host);
        if (ret != ESP_OK) {
            ESP_LOGE(TAG, "Failed to free SPI bus: %d", ret);
            return FPC_BEP_RESULT_INTERNAL_ERROR;
        }

        platform_bmlite_reset();

        gpio_reset_pin(p->pins->cs_n_pin);
        gpio_reset_pin(p->pins->miso_pin);
        gpio_reset_pin(p->pins->mosi_pin);
        gpio_reset_pin(p->pins->spi_clk_pin);
        gpio_reset_pin(p->pins->rst_pin);
        gpio_reset_pin(p->pins->irq_pin);
        
        p->pins = NULL;
    }

    pins = NULL;

    return FPC_BEP_RESULT_OK;
}

fpc_bep_result_t hal_board_init(void *params)
{
    
    console_initparams_t *p = (console_initparams_t *)params;

    if (!p || !p->pins) {
        ESP_LOGE(TAG, "Invalid init params");
        return FPC_BEP_RESULT_INTERNAL_ERROR;
    }

    pins = p->pins;
    
    if (p->iface == COM_INTERFACE) {
        ESP_LOGE(TAG, "UART Interface not supported!");
        return FPC_BEP_RESULT_INTERNAL_ERROR;
    }

    spi_bus_config_t buscfg = {
        .miso_io_num = pins->miso_pin,
        .mosi_io_num = pins->mosi_pin,
        .sclk_io_num = pins->spi_clk_pin,
        .quadwp_io_num = -1,
        .quadhd_io_num = -1,
        .max_transfer_sz = 2048,
    };

    spi_device_interface_config_t devcfg = {
        .mode = 0,
        .clock_speed_hz = p->baudrate,
        .spics_io_num = pins->cs_n_pin,
        .queue_size = 1,
    };
    
    esp_err_t ret = spi_bus_initialize(pins->spi_host, &buscfg, SPI_DMA_CH_AUTO);
    if (ret != ESP_OK) {
        spi_bus_free(pins->spi_host);
        pins = NULL;
        return FPC_BEP_RESULT_INTERNAL_ERROR;
    }

    ret = spi_bus_add_device(pins->spi_host, &devcfg, &spi_handle);
    if (ret != ESP_OK) {
        ESP_LOGE(TAG, "Failed to add SPI device");
        spi_bus_free(pins->spi_host);
        return FPC_BEP_RESULT_INTERNAL_ERROR;
    }

    // Init RST Pin
    gpio_config_t io_conf = {
        .pin_bit_mask = (1ULL << pins->rst_pin),
        .mode = GPIO_MODE_OUTPUT,
        .pull_up_en = GPIO_PULLUP_DISABLE,
        .pull_down_en = GPIO_PULLDOWN_DISABLE,
        .intr_type = GPIO_INTR_DISABLE,
    };

    ret = gpio_config(&io_conf);
    if (ret != ESP_OK) {
        return FPC_BEP_RESULT_INTERNAL_ERROR;
    }

    // Init IRQ Pin
    io_conf.pin_bit_mask = (1ULL << pins->irq_pin);
    io_conf.mode = GPIO_MODE_INPUT;
    
    ret = gpio_config(&io_conf);
    if (ret != ESP_OK) {
        return FPC_BEP_RESULT_INTERNAL_ERROR;
    }

    p->hcp_comm->read = platform_bmlite_spi_receive;
    p->hcp_comm->write = platform_bmlite_spi_send;
    p->hcp_comm->phy_rx_timeout = p->timeout;

    return FPC_BEP_RESULT_OK;
}

// THIS IS NULL
void hal_bmlite_reset(bool state)
{
    gpio_set_level(pins->rst_pin, state ? 0 : 1);  // Active Low
}

bool hal_bmlite_get_status(void)
{
    return gpio_get_level(pins->irq_pin) == 1;  // Active High
}

fpc_bep_result_t hal_bmlite_spi_write_read(uint8_t *write, uint8_t *read, size_t size, bool leave_cs_asserted)
{
    if (size == 0) {
        return FPC_BEP_RESULT_OK;
    }

    spi_transaction_t t = {
        .length = size * 8,
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

void hal_timebase_init(void) {} // Unnecessary on ESP32

hal_tick_t hal_timebase_get_tick(void)
{
    // microseconds -> milliseconds
    return esp_timer_get_time() / 1000;
}

void hal_timebase_busy_wait(uint32_t ms)
{
    vTaskDelay(pdMS_TO_TICKS(ms));
}

// Not using UART
size_t hal_bmlite_uart_write(const uint8_t *data, size_t size)
{
    return 0;
}

size_t hal_bmlite_uart_read(uint8_t *buff, size_t size)
{
    return 0;
}
