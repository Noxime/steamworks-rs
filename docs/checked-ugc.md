# Checked Workshop workflows

The checked UGC API keeps native result information and reports input, transport,
and Steam failures separately. Existing convenience APIs remain available.

## Result delivery and ownership

Controlled asynchronous methods return a `CallResultHandle`. Keep it alive while
you want the completion callback. Cancelling or dropping it stops delivery that
has not started; it does not undo a Steam operation or cancel an upload. A
cancelled operation can remain native-pending until Steam completes it. Keep that
work in your application's in-flight budget.

The callback pump runs completions on its caller's thread, outside the result
registry lock. A completion can start another request. Query resources remain
owned until completion or SDK shutdown, including when delivery is cancelled.
Retain upload content and preview files until the upload finishes or the SDK
shuts down. Account sessions, deadlines, pagination limits, and installation
transactions belong to the application.
Use one pump thread per pipe and do not recursively enter the manual-dispatch
pump from a completion; starting a new asynchronous request does not require
another pump invocation on that stack.

The handle holds a weak reference to the delivery state. Keep the `Client` or an
owning interface alive while pumping results. Use `is_native_pending()` to retain
cancelled work in a bounded retirement list until its native cleanup is done.

## Queries

Use checked configuration methods to handle invalid strings and native rejection
without panicking. Required tag groups mean OR within a group and AND between
groups. Empty groups are rejected. Steam performs filtering before pagination.

Strict detail and ID-only readers have different contracts. ID-only results must
not be validated as full detail records. A failed item must remain a failure,
rather than disappear from a list. The existing result count, total count, and
cache flag are available to decide whether a multi-page read is usable.

Local subscription counts are not an atomic server snapshot. Equal counts cannot
detect an equal-sized replacement, and missing locally cached IDs do not prove
that the user unsubscribed. Include locally disabled items when needed.

For `item_install_info_with_capacity`, allow spare capacity after the path's NUL
terminator. Some Steam clients report success even when truncating the path.
The checked reader rejects empty output and a terminator in the last byte;
an exactly fitting path must also be retried with a larger buffer because it
cannot be distinguished from truncation.

## Compatibility

Legacy asynchronous methods continue tracking completion even when their return
value is ignored. New controlled methods opt into handle-owned delivery. Checked
errors retain raw Steam result codes when supplied; a native `false` is reported
as rejection without inventing a more specific Steam error.

Legacy create/subscribe/unsubscribe/fetch/submit wrappers use the checked native
decoders internally. When Steam returns an invalid start handle, those wrappers
can now report an error synchronously instead of retaining an unfinishable call.
Legacy setters keep their existing error policy; use `try_*` setters for
recoverable errors. Legacy submit retains its previous NUL-note omission policy,
while `try_submit_with_handle` rejects that input. Unknown progress maps to the
legacy `Invalid` state; `progress_checked` returns the raw-value error.

## Using the API

```rust,no_run
use steamworks::{AppId, AppIDs, Client, UGCQueryType, UGCType};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let client = Client::init_app(AppId(480))?;
let pending = client.ugc()
    .query_all(UGCQueryType::RankedByLastUpdatedDate, UGCType::Items,
        AppIDs::ConsumerAppId(AppId(480)), 1)?
    .try_add_required_tag_group(&["category-a", "category-b"])?
    .try_set_allow_cached_response(0)?
    .fetch_ids_checked(|result| match result {
        Ok(page) => println!("{} of {} IDs", page.returned_count, page.total_count),
        Err(error) => eprintln!("query failed: {error}"),
    })?;
// Retain `pending` and pump client.process_callbacks(...) until completion.
// Dropping `pending` stops future delivery, not the Steam operation.
# Ok(())
# }
```

`examples/checked_workshop.rs` is read-only and takes its AppID and optional tag
groups from arguments. It exercises chained pages, a strict favorites ID page,
and reading the first item's vote. It never changes favorites, votes, or items.

## Validation

`cargo test --lib checked_ugc` runs the new offline regressions without a Steam
session. It does not execute the unrelated Steam-dependent legacy tests.

The native ABI test compares Rust sizes, alignments and consumed field offsets to
the bundled C++ headers. On Unix:

```sh
c++ -std=c++11 -I steamworks-sys/lib/steam/public tests/checked_ugc_layout.cpp -o target/checked-ugc-layout
STEAMWORKS_ABI_PROBE="$PWD/target/checked-ugc-layout" cargo test --test checked_ugc_abi -- --ignored
```

On Windows, compile the same probe in an MSVC developer environment and set
`STEAMWORKS_ABI_PROBE` to the resulting executable. Miri runs the pure Rust
regressions; neither Miri nor ABI checks replace live Steam API testing.
