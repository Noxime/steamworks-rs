use std::{path::Path, sync::mpsc::TryRecvError};
use std::ffi::{c_char, CString};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use steamworks::{Client, ClientManager, LeaderboardDataRequest, LeaderboardEntry, PublishedFileId, UploadScoreMethod, UGC};

fn main() {
    // create a client pair
    let client = Client::init().expect("Steam is not running or has not been detected");

    let lb = Arc::new(Mutex::new(None));
    let l = Arc::clone(&lb);
    client.user_stats().find_leaderboard("test",move |r| {
        println!("RESULT: {:?}", r);

        l.lock().unwrap().replace(r.unwrap().unwrap());
    });

    std::thread::sleep(std::time::Duration::from_millis(500));
    client.run_callbacks();

    let l = lb.lock().unwrap().take().unwrap(); // SteamLeaderboard_t

    let rs = client.remote_storage();

    rs.set_cloud_enabled_for_app(true);

    std::thread::sleep(std::time::Duration::from_millis(500));
    client.run_callbacks();

    println!("{}, {}", rs.is_cloud_enabled_for_account(), rs.is_cloud_enabled_for_app());


    // let f = rs.file("FILE_NAME_HERE");

    // println!("{}", f.write().write("123456789 ABC".as_bytes()).unwrap());

    let mut s = String::new();



    // f.read().read_to_string(&mut s).unwrap();

    println!("File data: {}", s);

    // client.test_new_fn(&l);

    let lbe: Arc<Mutex<Option<Vec<LeaderboardEntry>>>> = Arc::new(Mutex::new(None));
    let lbe2 = Arc::clone(&lbe);
    client.user_stats().download_leaderboard_entries(&l,LeaderboardDataRequest::Global,0,10,10,move |cb| {
        lbe2.lock().unwrap().replace(cb.unwrap());
    });

    std::thread::sleep(std::time::Duration::from_millis(500));
    client.run_callbacks();

    for entry in lbe.lock().unwrap().as_ref().unwrap().as_slice() {
        let file_content = client.download_ugc(entry.ugc);
        println!("ID: {:?},Score: {},UGC_handle: {:?},File content: {}", entry.user, entry.score, entry.ugc, file_content);
    }

    let mut s = String::new();
    // let f = client.remote_storage().file("FILE_NAME_HERE");

    // println!("CCC: {}, {}, {}", f.exists(), f.is_persisted(),f.timestamp());

    // f.read().read_to_string(&mut s).unwrap();

    // println!("File content: {}", s);




    std::thread::sleep(std::time::Duration::from_millis(500));

    // client.user_stats().upload_leaderboard_score(&l, UploadScoreMethod::ForceUpdate, 3, &[0;1], |r| {
    //     println!("Result: {:?}", r);
    // });

    std::thread::sleep(std::time::Duration::from_millis(500));

    client.run_callbacks();



    std::thread::sleep(std::time::Duration::from_millis(500));

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
