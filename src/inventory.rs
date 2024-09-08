use super::*;
use std::sync::Arc;

pub struct Inventory<Manager> {
    pub(crate) inventory: *mut sys::ISteamInventory,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> Inventory<Manager> {
    pub fn get_all_items(&self) -> Result<InventoryResult, InventoryError> {
        let mut result_handle = sys::k_SteamInventoryResultInvalid;
        unsafe {
            let success = sys::SteamAPI_ISteamInventory_GetAllItems(self.inventory, &mut result_handle);
            if success {
                Ok(InventoryResult::new(result_handle))
            } else {
                Err(InventoryError::OperationFailed)
            }
        }
    }

    pub fn get_result_items(&self, result_handle: InventoryResult) -> Result<Vec<InventoryItem>, InventoryError> {
        let mut items_count = 0;
        unsafe {
            let success = sys::SteamAPI_ISteamInventory_GetResultItems(
                self.inventory,
                result_handle.0,
                std::ptr::null_mut(),
                &mut items_count,
            );
            if !success {
                return Err(InventoryError::GetResultItemsFailed);
            }
        }
        let mut items_array: Vec<sys::SteamItemDetails_t> = Vec::with_capacity(items_count as usize);
        unsafe {
            let success = sys::SteamAPI_ISteamInventory_GetResultItems(
                self.inventory,
                result_handle.0,
                items_array.as_mut_ptr(),
                &mut items_count,
            );
            if success {
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
    use std::time::{Duration, Instant};
    use std::thread::sleep;
    use crate::sys::EResult;

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
    fn test_inventory_get_result_items() {
        let client = Client::init().unwrap();
        let inventory = client.inventory();
        let result = inventory.get_all_items();
        assert!(result.is_ok(), "Failed to get all items");

        let result_handle = result.unwrap();
        if result_handle.0 == sys::k_SteamInventoryResultInvalid {
            return;
        }

        let timeout_duration = Duration::from_secs(10);
        let polling_interval = Duration::from_millis(500);
        let start_time = Instant::now();
        let mut result_status;
        loop {
            result_status = unsafe {
                sys::SteamAPI_ISteamInventory_GetResultStatus(inventory.inventory, result_handle.0)
            };
            if result_status == EResult::k_EResultOK {
                break;
            } else if start_time.elapsed() > timeout_duration {
                panic!("Failed to retrieve inventory result within the timeout period.");
            }
            sleep(polling_interval);
        }

        let items = inventory.get_result_items(result_handle);
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
    }
}