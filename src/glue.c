#include "glue.h"
#include <stdlib.h>
#include <hdr_malloc.h>
#include <string.h>

int64_t hdr_rust_total_count(const struct hdr_histogram *h)
{
    return h->total_count;
}

int64_t hdr_rust_counts_len(const struct hdr_histogram *h)
{
    return h->counts_len;
}

struct hdr_histogram *hdr_rust_clone(const struct hdr_histogram *h)
{
    struct hdr_histogram *histogram;
    int64_t *counts;

    counts = (int64_t *)hdr_calloc((size_t)h->counts_len, sizeof(int64_t));
    if (!counts)
    {
        return NULL;
    }

    histogram = (struct hdr_histogram *)hdr_calloc(1, sizeof(struct hdr_histogram));
    if (!histogram)
    {
        hdr_free(counts);
        return NULL;
    }

    *histogram = *h;
    memcpy(counts, h->counts, sizeof(*counts) * h->counts_len);
    histogram->counts = counts;

    return histogram;
}
