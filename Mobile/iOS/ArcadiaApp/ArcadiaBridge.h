#pragma once
#include <stddef.h>
#include <stdint.h>

void arcadia_ios_start(uintptr_t metal_layer_ptr, const char* config_root);
void arcadia_ios_inject_touch(float x, float y, uint8_t phase);
void arcadia_ios_accessibility_node_count(size_t* out_count);
void arcadia_ios_accessibility_query_node(size_t index, float* out_rect4, uint64_t* out_traits,
    void* label_buf, size_t label_cap, void* hint_buf, size_t hint_cap,
    size_t* out_label_len, size_t* out_hint_len);
