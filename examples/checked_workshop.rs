//! Read-only exercise of chained queries and strict ID-only results.
//! cargo run --example checked_workshop -- APP_ID [tag-a,tag-b] [another-group]
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use steamworks::*;

type Completion = Arc<Mutex<Option<Result<(), String>>>>;
type Handles = Arc<Mutex<Vec<CallResultHandle>>>;
type FirstItem = Arc<Mutex<Option<PublishedFileId>>>;

fn request_page(
    client: Client,
    app: AppId,
    groups: Arc<Vec<Vec<String>>>,
    page: u32,
    handles: Handles,
    done: Completion,
    first_item: FirstItem,
) -> Result<(), UgcError> {
    let mut query = client
        .ugc()
        .query_all(
            UGCQueryType::RankedByLastUpdatedDate,
            UGCType::Items,
            AppIDs::Both {
                creator: app,
                consumer: app,
            },
            page,
        )
        .map_err(|_| UgcError::NativeRejected("create query"))?;
    for group in groups.iter() {
        query = query.try_add_required_tag_group(group)?;
    }
    let next_handles = handles.clone();
    let completion = done.clone();
    let handle = query
        .try_set_return_metadata(true)?
        .try_set_allow_cached_response(0)?
        .fetch_with_handle(move |result| {
            let result = result.map_err(|e| e.to_string()).and_then(|results| {
                println!(
                    "page {page}: {} returned / {} total, cached={}",
                    results.returned_results(),
                    results.total_results(),
                    results.was_cached()
                );
                for item in results.iter_checked() {
                    let item = item.map_err(|e| e.to_string())?;
                    first_item
                        .lock()
                        .unwrap()
                        .get_or_insert(item.published_file_id);
                }
                Ok(())
            });
            if result.is_ok() && page == 1 {
                // Registering the next CallResult from a completion must not deadlock.
                if let Err(e) = request_page(client, app, groups, 2, next_handles, done, first_item)
                {
                    *completion.lock().unwrap() = Some(Err(e.to_string()));
                }
            } else {
                *done.lock().unwrap() = Some(result);
            }
        })?;
    handles.lock().unwrap().push(handle);
    Ok(())
}

fn wait(
    client: &Client,
    done: &Completion,
    handles: &Handles,
) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        client.process_callbacks(|_| {});
        if let Some(result) = done.lock().unwrap().take() {
            return result.map_err(Into::into);
        }
        if Instant::now() >= deadline {
            for h in handles.lock().unwrap().iter() {
                h.cancel();
            }
            return Err("timed out; native operations may still be pending".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let app = AppId(
        args.next()
            .ok_or("usage: checked_workshop APP_ID [tag-group ...]")?
            .parse()?,
    );
    let groups = Arc::new(
        args.map(|g| g.split(',').map(str::to_owned).collect())
            .collect(),
    );
    let client = Client::init_app(app)?;
    if client.utils().app_id() != app {
        return Err("SDK AppID differs from requested AppID".into());
    }
    let handles: Handles = Arc::new(Mutex::new(Vec::new()));
    let done: Completion = Arc::new(Mutex::new(None));
    let first_item: FirstItem = Arc::new(Mutex::new(None));
    request_page(
        client.clone(),
        app,
        groups,
        1,
        handles.clone(),
        done.clone(),
        first_item.clone(),
    )?;
    wait(&client, &done, &handles)?;
    handles.lock().unwrap().clear();

    let complete = done.clone();
    let query = client.ugc().query_user(
        client.user().steam_id().account_id(),
        UserList::Favorited,
        UGCType::Items,
        UserListOrder::CreationOrderDesc,
        AppIDs::Both {
            creator: app,
            consumer: app,
        },
        1,
    )?;
    let ids = query
        .try_set_allow_cached_response(0)?
        .fetch_ids_checked(move |r| {
            *complete.lock().unwrap() = Some(
                r.map(|page| {
                    println!(
                        "favorites: {} returned / {} total, cached={}",
                        page.returned_count, page.total_count, page.was_cached
                    )
                })
                .map_err(|e| e.to_string()),
            );
        })?;
    handles.lock().unwrap().push(ids);
    wait(&client, &done, &handles)?;
    if let Some(item) = *first_item.lock().unwrap() {
        let completion = done.clone();
        let vote = client.ugc().get_user_item_vote(item, move |r| {
            *completion.lock().unwrap() = Some(
                r.map(|vote| {
                    println!(
                        "vote: up={}, down={}, skipped={}",
                        vote.voted_up, vote.voted_down, vote.vote_skipped
                    );
                })
                .map_err(|e| e.to_string()),
            );
        })?;
        handles.lock().unwrap().push(vote);
        wait(&client, &done, &handles)?;
    }
    Ok(())
}
