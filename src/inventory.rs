use super::*;
use std::sync::Arc;
use crate::sys;

pub struct Inventory<Manager> {
    pub(crate) inventory: *mut sys::ISteamInventory,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamInventoryResultReady {
    pub handle: sys::SteamInventoryResult_t,
    pub result: Result<(), SteamError>,
}

unsafe impl Callback for SteamInventoryResultReady {
    const ID: i32 = sys::SteamInventoryResultReady_t_k_iCallback as i32;
    const SIZE: i32 = std::mem::size_of::<sys::SteamInventoryResultReady_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let status = &*(raw as *mut sys::SteamInventoryResultReady_t);

        Self {
            handle: status.m_handle,
            result: match status.m_result {
                sys::EResult::k_EResultOK => Ok(()),
                _ => Err(SteamError::from(status.m_result)),
            },
        }
    }
}

impl<Manager> Inventory<Manager> {
    pub fn get_all_items(&self) -> Result<InventoryResult, InventoryError> {
        let mut result_handle = sys::k_SteamInventoryResultInvalid;
        unsafe {
            if sys::SteamAPI_ISteamInventory_GetAllItems(self.inventory, &mut result_handle) {
                Ok(InventoryResult::new(result_handle))
            } else {
                Err(InventoryError::OperationFailed)
            }
        }
    }

    pub fn get_result_items(&self, result_handle: InventoryResult) -> Result<Vec<InventoryItem>, InventoryError> {
        let mut items_count = 0;
        unsafe {
            if !sys::SteamAPI_ISteamInventory_GetResultItems(
                self.inventory,
                result_handle.0,
                std::ptr::null_mut(),
                &mut items_count,
            ) {
                return Err(InventoryError::GetResultItemsFailed);
            }
        }

        let mut items_array: Vec<sys::SteamItemDetails_t> = Vec::with_capacity(items_count as usize);
        unsafe {
            if sys::SteamAPI_ISteamInventory_GetResultItems(
                self.inventory,
                result_handle.0,
                items_array.as_mut_ptr(),
                &mut items_count,
            ) {
                items_array.set_len(items_count as usize);
                let items = items_array.into_iter().map(|details| InventoryItem {
                    item_id: ItemInstanceId(details.m_itemId),
                    definition: ItemDefId(details.m_iDefinition),
                    quantity: details.m_unQuantity,
                    flags: details.m_unFlags,
                }).collect();
                Ok(items)
            } else {
                Err(InventoryError::GetResultItemsFailed)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct InventoryResult(pub(crate) sys::SteamInventoryResult_t);

impl InventoryResult {
    pub fn new(result_handle: sys::SteamInventoryResult_t) -> Self {
        InventoryResult(result_handle)
    }

    pub fn handle(&self) -> i32 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct InventoryItem {
    pub item_id: ItemInstanceId,
    pub definition: ItemDefId,
    pub quantity: u16,
    pub flags: u16,
}

#[derive(Clone, Debug)]
pub struct ItemInstanceId(pub u64);

#[derive(Clone, Debug)]
pub struct ItemDefId(pub i32);

#[derive(Debug, Error)]
pub enum InventoryError {
    #[error("The inventory operation failed")]
    OperationFailed,
    #[error("Failed to retrieve result items")]
    GetResultItemsFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::sync::{Arc, Mutex};

    #[test]
    #[serial]
    fn test_inventory_get_all_items() {
        let client = Client::init().unwrap();
        let inventory = client.inventory();
        let result = inventory.get_all_items();
        assert!(result.is_ok(), "Failed to get all items");
    }

    #[test]
    #[serial]
    fn test_inventory_get_result_items_with_callback() {
        let client = Client::init().unwrap();
        let inventory = Arc::new(Mutex::new(client.inventory()));
        let result_handle = Arc::new(Mutex::new(None));

        let result_handle_clone = Arc::clone(&result_handle);
        let _cb = client.register_callback(move |val: SteamInventoryResultReady| {
            match val.result {
                Ok(()) => {
                    let mut result_handle_lock = result_handle_clone.lock().unwrap();
                    *result_handle_lock = Some(InventoryResult(val.handle));
                },
                Err(e) => {
                    panic!("Inventory result failed: {:?}", e);
                },
            }
        });

        let result = inventory.lock().unwrap().get_all_items();
        println!("{:?}", result);
        assert!(result.is_ok(), "Failed to get all items");
        for _ in 0..50 {
            client.run_callbacks();
            std::thread::sleep(std::time::Duration::from_millis(100));

            let result_handle_lock = result_handle.lock().unwrap();
            if let Some(handle) = &*result_handle_lock {
                let items = inventory.lock().unwrap().get_result_items(handle.clone());
                assert!(items.is_ok(), "Failed to retrieve result items");

                let items = items.unwrap();
                println!("Total items count: {}", items.len());
                if items.is_empty() {
                    println!("No items found in the inventory.");
                } else {
                    for (index, item) in items.iter().enumerate() {
                        println!(
                            "Item {} - ID: {}, Definition: {}, Quantity: {}, Flags: {}",
                            index + 1,
                            item.item_id.0,
                            item.definition.0,
                            item.quantity,
                            item.flags
                        );
                    }
                }
                return;
            }
        }
        panic!("Timed out waiting for inventory result");
    }
}