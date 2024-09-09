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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamInventoryFullUpdate {
    pub handle: sys::SteamInventoryResult_t,
}

unsafe impl Callback for SteamInventoryFullUpdate {
    const ID: i32 = sys::SteamInventoryFullUpdate_t_k_iCallback as i32;
    const SIZE: i32 = std::mem::size_of::<sys::SteamInventoryFullUpdate_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let status = &*(raw as *mut sys::SteamInventoryFullUpdate_t);

        Self {
            handle: status.m_handle,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamInventoryStartPurchaseResult {
    pub result: Result<(), SteamError>,
    pub order_id: u64,
    pub trans_id: u64,
}

unsafe impl Callback for SteamInventoryStartPurchaseResult {
    const ID: i32 = sys::SteamInventoryStartPurchaseResult_t_k_iCallback as i32;
    const SIZE: i32 = std::mem::size_of::<sys::SteamInventoryStartPurchaseResult_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let status = &*(raw as *mut sys::SteamInventoryStartPurchaseResult_t);

        Self {
            result: match status.m_result {
                sys::EResult::k_EResultOK => Ok(()),
                _ => Err(SteamError::from(status.m_result)),
            },
            order_id: status.m_ulOrderID,
            trans_id: status.m_ulTransID,
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

    pub fn start_purchase(
        &self,
        item_defs: &[ItemDefId],
        quantities: &[u32],
    ) -> Result<sys::SteamAPICall_t, InventoryError> {
        if item_defs.len() != quantities.len() {
            return Err(InventoryError::InvalidInput);
        }

        let item_def_ids: Vec<sys::SteamItemDef_t> = item_defs.iter().map(|id| id.0).collect();
        let result = unsafe {
            sys::SteamAPI_ISteamInventory_StartPurchase(
                self.inventory,
                item_def_ids.as_ptr(),
                quantities.as_ptr(),
                item_defs.len() as u32,
            )
        };

        if result == sys::k_uAPICallInvalid {
            Err(InventoryError::OperationFailed)
        } else {
            Ok(result)
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
    #[error("Invalid input")]
    InvalidInput,
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

    #[test]
    #[serial]
    fn test_inventory_full_update() {
        let client = Client::init().unwrap();
        let inventory = Arc::new(Mutex::new(client.inventory()));
        let result_handle = Arc::new(Mutex::new(None));

        let result_handle_clone = Arc::clone(&result_handle);
        let _cb = client.register_callback(move |val: SteamInventoryFullUpdate| {
            let mut result_handle_lock = result_handle_clone.lock().unwrap();
            *result_handle_lock = Some(InventoryResult(val.handle));
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
                println!("Total items count after full update: {}", items.len());
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
        panic!("Timed out waiting for full inventory update");
    }

    #[test]
    #[serial]
    fn test_inventory_start_purchase_with_callback() {
        let client = Client::init().unwrap();
        let inventory = Arc::new(Mutex::new(client.inventory()));
        let purchase_result = Arc::new(Mutex::new(None));

        let purchase_result_clone = Arc::clone(&purchase_result);
        let _cb = client.register_callback(move |val: SteamInventoryStartPurchaseResult| {
            let mut purchase_result_lock = purchase_result_clone.lock().unwrap();
            *purchase_result_lock = Some(val);
        });

        // Example item definition IDs and quantities
        let item_defs = vec![ItemDefId(634)];  // Replace with valid item definition IDs
        let quantities = vec![1];  // Replace with desired quantities

        let result = inventory.lock().unwrap().start_purchase(&item_defs, &quantities);
        assert!(result.is_ok(), "Failed to start purchase");
        let api_call = result.unwrap();
        println!("SteamAPICall_t for purchase: {}", api_call);

        // Wait for the callback to be received
        for _ in 0..50 {
            client.run_callbacks();
            std::thread::sleep(std::time::Duration::from_millis(100));

            let purchase_result_lock = purchase_result.lock().unwrap();
            if let Some(purchase) = &*purchase_result_lock {
                match &purchase.result {
                    Ok(()) => {
                        println!(
                            "Purchase successful! Order ID: {}, Transaction ID: {}",
                            purchase.order_id, purchase.trans_id
                        );
                    }
                    Err(e) => {
                        println!("Purchase failed: {:?}", e);
                    }
                }
                return;
            }
        }
        panic!("Timed out waiting for purchase result");
    }

}