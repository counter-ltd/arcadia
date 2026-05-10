#pragma once
#include <stdint.h>

void arcadia_ios_start(uintptr_t metal_layer_ptr, const char* config_root);
void arcadia_ios_inject_touch(float x, float y, uint8_t phase);
