/*
 * Copyright (c) 2020 Andrey Perminov <andrey.ppp@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#ifndef CONSOLE_PARAMS_H
#define CONSOLE_PARAMS_H

#include <stdint.h>
#include <stdbool.h>

#include "fpc_bep_types.h"
#include "hcp_tiny.h"

#include <driver/spi_master.h>
#include <driver/gpio.h>

typedef enum {
   COM_INTERFACE = 0,
   SPI_INTERFACE
} interface_t;

typedef struct {
   spi_host_device_t spi_host;
   gpio_num_t cs_n_pin;
   gpio_num_t miso_pin;
   gpio_num_t rst_pin;
   gpio_num_t mosi_pin;
   gpio_num_t irq_pin;
   gpio_num_t spi_clk_pin;
} pin_config_t;

typedef struct {
   interface_t iface;
   char *port;
   uint32_t baudrate;
   uint32_t timeout;
   HCP_comm_t *hcp_comm;
   pin_config_t *pins;
} console_initparams_t;

#endif