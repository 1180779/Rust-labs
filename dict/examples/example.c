#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct RedBlackTree RedBlackTree;

extern RedBlackTree* rbt_new();
extern void rbt_free(RedBlackTree* tree);
extern bool rbt_insert(RedBlackTree* tree, uint64_t key, const char* value);
extern const char* rbt_find(const RedBlackTree* tree, uint64_t key);
extern bool rbt_contains(const RedBlackTree* tree, uint64_t key);
extern bool rbt_remove(RedBlackTree* tree, uint64_t key);
extern const char* rbt_minimum(const RedBlackTree* tree);
extern const char* rbt_maximum(const RedBlackTree* tree);

int main() {
    printf("=== Red-Black Tree Dictionary Example (C) ===\n");

    printf("\nCreating dictionary\n");
    RedBlackTree* d = rbt_new();
    if (d == NULL) {
        printf("Dictionary creation failed. Exiting\n");
        return 1;
    }

    rbt_insert(d, 1, "a");
    rbt_insert(d, 12, "ab");
    rbt_insert(d, 123, "abc");
    rbt_insert(d, 1234, "abcd");
    printf("Inserted keys: 1, 12, 123, 1234\n");

    printf("\nChecking if keys exist:\n");
    const uint64_t keys_to_check[] = {1, 12, 123, 1234, 2, 5, 56, 79, 1239};
    const size_t num_keys = sizeof(keys_to_check) / sizeof(keys_to_check[0]);

    for (size_t i = 0; i < num_keys; ++i) {
        const uint64_t key = keys_to_check[i];
        const bool exists = rbt_contains(d, key);
        printf("\tContains key %5lu %s\n", key, exists ? "true" : "false");
    }

    printf("\nFinding values:\n");
    for (size_t i = 0; i < num_keys; ++i) {
        const uint64_t key = keys_to_check[i];
        const char* val = rbt_find(d, key);
        if (val) {
            printf("\t%-10s %5lu %s\n", "Found", key, val);
        } else {
            printf("\t%-10s %5lu\n", "Not Found", key);
        }
    }

    printf("\nGet minimum\n");
    const char* min_val = rbt_minimum(d);
    if (min_val) {
        printf("\t%-15s %s\n", "Minimum", min_val);
    } else {
        printf("\t%-15s\n", "Dict empty");
    }

    printf("\nGet maximum\n");
    const char* max_val = rbt_maximum(d);
    if (max_val) {
        printf("\t%-15s %s\n", "Maximum", max_val);
    } else {
        printf("\t%-15s\n", "Dict empty");
    }

    printf("\nDeleting keys\n");
    for (size_t i = 0; i < num_keys; ++i) {
        const uint64_t key = keys_to_check[i];
        const bool removed = rbt_remove(d, key);
        if (removed) {
            printf("\t%-15s %5lu\n", "Removed value", key);
        } else {
            printf("\t%-15s %5lu\n", "Was not present", key);
        }
    }

    rbt_free(d);
    return 0;
}