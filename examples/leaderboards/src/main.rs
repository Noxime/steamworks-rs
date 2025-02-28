use std::sync::mpsc::{channel, Sender};
use steamworks::{Client, LeaderboardDataRequest, UploadScoreMethod};

fn main() {
    // create a client pair
    let client = Client::init().expect("Steam is not running or has not been detected");

    // Get a suitable leaderboard to use for this example
    let leaderboard_to_use = process_client_callback(&client, |client,sender| {
        client.user_stats().find_leaderboard("test",move |r| {
            sender.send(r.unwrap().unwrap()).unwrap();
        });
    });

    // Try changing this if statement to true and false and re-running the example to see it still be able to read the data!
    if true {
        let lb_clone = leaderboard_to_use.clone();
        let l_ugc = process_client_callback(&client, move |client,sender| {

            client.user_stats().upload_leaderboard_score(&lb_clone, UploadScoreMethod::ForceUpdate, 5, &[0;1], |r| {
                println!("Result: {:?}", r);
            });

            client.remote_storage().file_share("this_is_a_new_filename", "new test data 123",move |a| {
                sender.send(a.unwrap()).unwrap();
            });
        });

        client.user_stats().attach_leaderboard_ugc(l_ugc, &leaderboard_to_use, |res| {
            println!("Attached ugc: {:?}", res);
        });
    }

    // The data should be downloaded in "C:\Program Files (x86)\Steam\userdata\<YOUR STEAM ID>\<APP ID>\remote"
    // Try deleting the files and running this example again!
    let l_entries = process_client_callback(&client, move |client,sender| {
        client.user_stats().download_leaderboard_entries(&leaderboard_to_use, LeaderboardDataRequest::Global, 0, 10, 10, move |cb| {
            sender.send(cb.unwrap()).unwrap();
        });
    });

    // Display all the leaderboard entries and their file content if they have UGC
    for entry in l_entries {
        let entry_clone = entry.clone();
        let file_content = {
            let ugc_result = process_client_callback(&client, move |client, sender| {
                client.remote_storage().download_ugc(&entry, move |file_content| {
                    sender.send(file_content).unwrap();
                });
            }).unwrap();

            client.remote_storage().ugc_read(ugc_result)
        };
        println!("ID: {:?},Score: {},UGC_handle: {:?},File content: {}", entry_clone.user, entry_clone.score, entry_clone.ugc, file_content);
    }

    // create a thread for callbacks
    // if you have an active loop (like in a game), you can skip this and just run the callbacks on update
    let callback_thread = std::thread::spawn(move || {
        loop {
            // run callbacks
            client.run_callbacks();
            std::thread::sleep(std::time::Duration::from_millis(5000));

        }
    });
}

fn process_client_callback<F,T>(client: &Client, client_function: F) -> T
where F: FnOnce(&Client, Sender<T>) + 'static + Send, {
    let (tx,rx) = channel::<T>();

    client_function(client, tx);

    loop {
        client.run_callbacks();

        match rx.try_recv() {
            Ok(l) => {
                break l
            }
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }
}