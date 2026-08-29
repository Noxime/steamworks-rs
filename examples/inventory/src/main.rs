use std::sync::mpsc;

use steamworks::{Client, ItemDefId};

/// Lists the current user's inventory, then loads the app's item definitions.
///
/// Run with a `steam_appid.txt` for an app that has the Inventory Service
/// enabled; Spacewar (480) works for listing, though its inventory is empty.
fn main() {
    let client = Client::init().expect("Steam must be running");
    let inventory = client.inventory();

    let (tx, rx) = mpsc::channel();

    inventory.get_all_items(move |result| {
        match result {
            Ok(result) => {
                let items = result.items();
                println!("{} item(s) at t={}", items.len(), result.timestamp());
                for item in items {
                    println!(
                        "  #{} def {} x{} flags {:?}",
                        item.item_id.0, item.definition.0, item.quantity, item.flags
                    );
                }
            }
            Err(err) => println!("get_all_items failed: {err:?}"),
        }
        tx.send(()).unwrap();
    });

    // Definitions arrive through the InventoryDefinitionUpdate callback.
    let defs_ready = {
        let (tx, rx) = mpsc::channel();
        let handle = client.register_callback(move |_: steamworks::InventoryDefinitionUpdate| {
            tx.send(()).ok();
        });
        inventory.load_item_definitions();
        (handle, rx)
    };

    let mut done = false;
    let mut defs_done = false;
    while !(done && defs_done) {
        client.run_callbacks();
        if rx.try_recv().is_ok() {
            done = true;
        }
        if defs_ready.1.try_recv().is_ok() {
            defs_done = true;
            if let Some(ids) = inventory.item_definition_ids() {
                println!("{} definition(s)", ids.len());
                for id in ids.iter().take(5) {
                    let name = inventory.item_definition_property(*id, Some("name"));
                    println!("  {:?}: {}", ItemDefId(id.0), name.unwrap_or_default());
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
