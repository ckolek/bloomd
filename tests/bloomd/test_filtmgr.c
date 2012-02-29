#include <check.h>
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <fcntl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <errno.h>
#include <dirent.h>
#include "config.h"
#include "filter.h"
#include "filter_manager.h"

START_TEST(test_mgr_init_destroy)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

START_TEST(test_mgr_create_drop)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_create_filter(mgr, "foo1", NULL);
    fail_unless(res == 0);

    res = filtmgr_drop_filter(mgr, "foo1");
    fail_unless(res == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

START_TEST(test_mgr_create_double_drop)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_create_filter(mgr, "dub1", NULL);
    fail_unless(res == 0);

    res = filtmgr_drop_filter(mgr, "dub1");
    fail_unless(res == 0);

    res = filtmgr_drop_filter(mgr, "dub1");
    fail_unless(res == -1);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

START_TEST(test_mgr_list)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_create_filter(mgr, "bar1", NULL);
    fail_unless(res == 0);
    res = filtmgr_create_filter(mgr, "bar2", NULL);
    fail_unless(res == 0);

    bloom_filter_list_head *head;
    res = filtmgr_list_filters(mgr, &head);
    fail_unless(res == 0);
    fail_unless(head->size == 2);

    int has_bar1 = 0;
    int has_bar2 = 0;

    bloom_filter_list *node = head->head;
    while (node) {
        if (strcmp(node->filter_name, "bar1") == 0)
            has_bar1 = 1;
        else if (strcmp(node->filter_name, "bar2") == 0)
            has_bar2 = 1;
        node = node->next;
    }
    fail_unless(has_bar1);
    fail_unless(has_bar2);

    res = filtmgr_drop_filter(mgr, "bar1");
    fail_unless(res == 0);
    res = filtmgr_drop_filter(mgr, "bar2");
    fail_unless(res == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

START_TEST(test_mgr_list_no_filters)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    bloom_filter_list_head *head;
    res = filtmgr_list_filters(mgr, &head);
    fail_unless(res == 0);
    fail_unless(head->size == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST


START_TEST(test_mgr_add_check_keys)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_create_filter(mgr, "zab1", NULL);
    fail_unless(res == 0);

    char *keys[] = {"hey","there","person"};
    char result[] = {0, 0, 0};
    res = filtmgr_set_keys(mgr, "zab1", (char**)&keys, 3, (char*)&result);
    fail_unless(res == 0);
    fail_unless(result[0]);
    fail_unless(result[1]);
    fail_unless(result[2]);

    for (int i=0;i<3;i++) result[i] = 0;
    res = filtmgr_check_keys(mgr, "zab1", (char**)&keys, 3, (char*)&result);
    fail_unless(res == 0);
    fail_unless(result[0]);
    fail_unless(result[1]);
    fail_unless(result[2]);

    res = filtmgr_drop_filter(mgr, "zab1");
    fail_unless(res == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

START_TEST(test_mgr_check_no_keys)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_create_filter(mgr, "zab2", NULL);
    fail_unless(res == 0);

    char *keys[] = {"hey","there","person"};
    char result[] = {1, 1, 1};
    res = filtmgr_check_keys(mgr, "zab2", (char**)&keys, 3, (char*)&result);
    fail_unless(res == 0);
    fail_unless(!result[0]);
    fail_unless(!result[1]);
    fail_unless(!result[2]);

    res = filtmgr_drop_filter(mgr, "zab2");
    fail_unless(res == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

START_TEST(test_mgr_add_check_no_filter)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    char *keys[] = {"hey","there","person"};
    char result[] = {0, 0, 0};
    res = filtmgr_set_keys(mgr, "noop1", (char**)&keys, 3, (char*)&result);
    fail_unless(res == -1);

    for (int i=0;i<3;i++) result[i] = 0;
    res = filtmgr_check_keys(mgr, "noop1", (char**)&keys, 3, (char*)&result);
    fail_unless(res == -1);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

/* Flush */
START_TEST(test_mgr_flush_no_filter)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_flush_filter(mgr, "noop1");
    fail_unless(res == -1);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

START_TEST(test_mgr_flush)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_create_filter(mgr, "zab3", NULL);
    fail_unless(res == 0);

    res = filtmgr_flush_filter(mgr, "zab3");
    fail_unless(res == 0);

    res = filtmgr_drop_filter(mgr, "zab3");
    fail_unless(res == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

/* Unmap */
START_TEST(test_mgr_unmap_no_filter)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_unmap_filter(mgr, "noop2");
    fail_unless(res == -1);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

START_TEST(test_mgr_unmap)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_create_filter(mgr, "zab4", NULL);
    fail_unless(res == 0);

    res = filtmgr_unmap_filter(mgr, "zab4");
    fail_unless(res == 0);

    res = filtmgr_drop_filter(mgr, "zab4");
    fail_unless(res == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

START_TEST(test_mgr_unmap_add_keys)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_create_filter(mgr, "zab5", NULL);
    fail_unless(res == 0);

    res = filtmgr_unmap_filter(mgr, "zab5");
    fail_unless(res == 0);

    // Try to add keys now
    char *keys[] = {"hey","there","person"};
    char result[] = {0, 0, 0};
    res = filtmgr_set_keys(mgr, "zab5", (char**)&keys, 3, (char*)&result);
    fail_unless(res == 0);
    fail_unless(result[0]);
    fail_unless(result[1]);
    fail_unless(result[2]);

    res = filtmgr_drop_filter(mgr, "zab5");
    fail_unless(res == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

/* List Cold */
START_TEST(test_mgr_list_cold_no_filters)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    bloom_filter_list_head *head;
    res = filtmgr_list_cold_filters(mgr, &head);
    fail_unless(res == 0);
    fail_unless(head->size == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

START_TEST(test_mgr_list_cold)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_create_filter(mgr, "zab6", NULL);
    fail_unless(res == 0);
    res = filtmgr_create_filter(mgr, "zab7", NULL);
    fail_unless(res == 0);

    bloom_filter_list_head *head;
    res = filtmgr_list_cold_filters(mgr, &head);
    fail_unless(res == 0);
    fail_unless(head->size == 0);

    // Check the keys in one, so that it stays hot
    char *keys[] = {"hey","there","person"};
    char result[] = {0, 0, 0};
    res = filtmgr_set_keys(mgr, "zab6", (char**)&keys, 3, (char*)&result);
    fail_unless(res == 0);
    fail_unless(result[0]);
    fail_unless(result[1]);
    fail_unless(result[2]);

    // Check cold again
    res = filtmgr_list_cold_filters(mgr, &head);
    fail_unless(res == 0);
    fail_unless(head->size == 1);

    int has_zab6 = 0;
    int has_zab7 = 0;

    bloom_filter_list *node = head->head;
    while (node) {
        if (strcmp(node->filter_name, "zab6") == 0)
            has_zab6 = 1;
        else if (strcmp(node->filter_name, "zab7") == 0)
            has_zab7 = 1;
        node = node->next;
    }
    fail_unless(!has_zab6);
    fail_unless(has_zab7);

    res = filtmgr_drop_filter(mgr, "zab6");
    fail_unless(res == 0);
    res = filtmgr_drop_filter(mgr, "zab7");
    fail_unless(res == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

/* Unmap in memory */
START_TEST(test_mgr_unmap_in_mem)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);
    config.in_memory = 1;

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    res = filtmgr_create_filter(mgr, "mem1", NULL);
    fail_unless(res == 0);

    // Try to add keys now
    char *keys[] = {"hey","there","person"};
    char result[] = {0, 0, 0};
    res = filtmgr_set_keys(mgr, "mem1", (char**)&keys, 3, (char*)&result);
    fail_unless(res == 0);
    fail_unless(result[0]);
    fail_unless(result[1]);
    fail_unless(result[2]);

    res = filtmgr_unmap_filter(mgr, "mem1");
    fail_unless(res == 0);

    // Try to add keys now
    for (int i=0;i<3;i++) result[i] = 0;
    res = filtmgr_check_keys(mgr, "mem1", (char**)&keys, 3, (char*)&result);
    fail_unless(res == 0);
    fail_unless(result[0]);
    fail_unless(result[1]);
    fail_unless(result[2]);

    res = filtmgr_drop_filter(mgr, "mem1");
    fail_unless(res == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

/* Custom config */
START_TEST(test_mgr_create_custom_config)
{
    bloom_config config;
    int res = config_from_filename(NULL, &config);
    fail_unless(res == 0);

    bloom_filtmgr *mgr;
    res = init_filter_manager(&config, &mgr);
    fail_unless(res == 0);

    // Custom config
    bloom_config *custom = malloc(sizeof(bloom_config));
    memcpy(custom, &config, sizeof(bloom_config));
    custom->in_memory = 1;

    res = filtmgr_create_filter(mgr, "custom1", custom);
    fail_unless(res == 0);

    res = filtmgr_drop_filter(mgr, "custom1");
    fail_unless(res == 0);

    res = destroy_filter_manager(mgr);
    fail_unless(res == 0);
}
END_TEST

/* Scale up */


/* Close & Restore */


