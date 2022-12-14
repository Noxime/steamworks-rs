use std::{path::Path, sync::mpsc::TryRecvError};

use steamworks::{Client, ClientManager, PublishedFileId, UGC};

fn create_item(ugc: &UGC<ClientManager>) {
    // creating a new workshop item
    // make sure you change the appid to the specified game
    ugc.create_item(480, steamworks::FileType::Community, |create_result| {
        // handle the result
        match create_result {
            Ok((published_id, needs_to_agree_to_terms)) => {
                // if the user needs to agree to the terms of use, they will need to do that before you can upload any files
                // in any case, make sure you save the published_id somewhere, like a manifest file.
                // it is needed for all further calls
                if needs_to_agree_to_terms {
                    println!(
                        "You need to agree to the terms of use before you can upload any files"
                    );
                } else {
                    println!("Published item with id {}", published_id);
                }
            }
            Err(e) => {
                // an error occurred, usually because the app is not authorized to create items
                // or the user is banned from the community
                println!("Error creating workshop item: {:?}", e);
            }
        }
    });
}

fn upload_item_content(ugc: &UGC<ClientManager>, published_id: PublishedFileId) {
    // uploading the content of the workshop item
    // this process uses a builder pattern to set properties of the item
    // mandatory properties are:
    // - title
    // - description
    // - preview_path
    // - content_path
    // - visibility
    // after setting the properties, call .submit() to start uploading the item
    // this function is unique in that it returns a handle to the upload, which can be used to
    // monitor the progress of the upload and needs a closure to be called when the upload is done
    // in this example, the watch handle is ignored for simplicity
    //
    // notes:
    // - once an upload is started, it cannot be cancelled!
    // - content_path is the path to a folder which houses the content you wish to upload
    let upload_handle = ugc
        .start_item_update(480, published_id)
        .content_path("/absolute/path/to/content")
        .preview_path(Path::new("/absolute/path/to/preview.png"))
        .title("Item title")
        .description("Item description")
        .tags([])
        .visibility(steamworks::PublishedFileVisibility::Public)
        .submit(Some("My changenotes"), |upload_result| {
            // handle the result
            match upload_result {
                Ok((published_id, needs_to_agree_to_terms)) => {
                    if needs_to_agree_to_terms {
                        // as stated in the create_item function, if the user needs to agree to the terms of use,
                        // the upload did NOT succeed, despite the result being Ok
                        println!(
                            "You need to agree to the terms of use before you can upload any files"
                        );
                    } else {
                        // this is the definite indicator that an item was uploaded successfully
                        // the watch handle is NOT an accurate indicator whether the upload is done
                        // the progress on the other hand IS accurate and can simply be used to monitor the upload
                        println!("Uploaded item with id {}", published_id);
                    }
                }
                Err(e) => {
                    // the upload failed
                    // the exact reason can be found in the error type
                    println!("Error uploading item: {:?}", e);
                }
            }
        });
}

fn delete_item(ugc: &UGC<ClientManager>, published_id: PublishedFileId) {
    // deleting an item
    ugc.delete_item(published_id, |delete_result| {
        match delete_result {
            Ok(()) => {
                // item has been deleted
                println!("Deleted item with id {}", published_id);
            }
            Err(e) => {
                // the item could not be deleted
                // usually because it is not owned by the user or it doesn't exist in the first place
                println!("Error deleting item: {:?}", e);
            }
        }
    })
}

fn main() {
    // create a client pair
    let (client, single) = Client::init().expect("Steam is not running or has not been detected");

    // create a channel to communicate with the upcoming callback thread
    // this is technically not *needed* but it is cleaner in order to properly exit the thread
    let (tx, rx) = std::sync::mpsc::channel();
    // create a thread for callbacks
    // if you have an active loop (like in a game), you can skip this and just run the callbacks on update
    let callback_thread = std::thread::spawn(move || {
        loop {
            // run callbacks
            single.run_callbacks();
            std::thread::sleep(std::time::Duration::from_millis(100));

            // check if the channel is closed or if there is a message
            // end the thread if either is true
            match rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {}
            }
        }
    });

    // get a handle to Steam's UGC module (user-generated content)
    let ugc = client.ugc();
    create_item(&ugc);

    // only do this once you received a successful callback for creating the item, else this WILL fail!
    // and fill the published file ID with an actual ID received from the SteamAPI!
    // as said above, for example from your manifest file
    upload_item_content(&ugc, PublishedFileId(413));

    // like above, also only do this with a valid published file ID
    delete_item(&ugc, PublishedFileId(413));

    // close the channel and wait for the callback thread to end
    tx.send(())
        .expect("Failed to send message to callback thread");
    callback_thread
        .join()
        .expect("Failed to join callback thread");

    Ok(())
}
