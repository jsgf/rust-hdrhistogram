#include <stdint.h>
#include <hdr_histogram.h>

#ifdef __cplusplus
extern "C" {
#endif

extern int64_t hdr_rust_total_count(const struct hdr_histogram *h);
extern int64_t hdr_rust_counts_len(const struct hdr_histogram *h);
extern struct hdr_histogram *hdr_rust_clone(const struct hdr_histogram *h);

#ifdef __cplusplus
}
#endif
