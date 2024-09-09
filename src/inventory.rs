use super::*;
use std::sync::Arc;
use crate::sys;

/// Represents the result of an inventory operation, ready to be processed.
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

/// Represents a full update event for the Steam inventory.
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

/// Provides access to the Steam inventory interface.
pub struct Inventory<Manager> {
    pub(crate) inventory: *mut sys::ISteamInventory,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> Inventory<Manager> {
    /// Retrieves all items in the user's Steam inventory.
    pub fn get_all_items(&self) -> Result<sys::SteamInventoryResult_t, InventoryError> {
        let mut result_handle = sys::k_SteamInventoryResultInvalid;
        unsafe {
            if sys::SteamAPI_ISteamInventory_GetAllItems(self.inventory, &mut result_handle) {
                Ok(result_handle)
            } else {
                Err(InventoryError::OperationFailed)
            }
        }
    }

    /// Retrieves the detailed list of items from the inventory given a result handle.
    pub fn get_result_items(&self, result_handle: sys::SteamInventoryResult_t) -> Result<Vec<SteamItemDetails>, InventoryError> {
        let mut items_count = 0;
        unsafe {
            if !sys::SteamAPI_ISteamInventory_GetResultItems(
                self.inventory,
                result_handle,
                std::ptr::null_mut(),
                &mut items_count,
            ) {
                return Err(InventoryError::GetResultItemsFailed);
            }

            let mut items_array: Vec<sys::SteamItemDetails_t> = Vec::with_capacity(items_count as usize);
            if sys::SteamAPI_ISteamInventory_GetResultItems(
                self.inventory,
                result_handle,
                items_array.as_mut_ptr(),
                &mut items_count,
            ) {
                items_array.set_len(items_count as usize);
                let items = items_array.into_iter().map(|details| SteamItemDetails {
                    item_id: SteamItemInstanceID(details.m_itemId),
                    definition: SteamItemDef(details.m_iDefinition),
                    quantity: details.m_unQuantity,
                    flags: details.m_unFlags,
                }).collect();
                self.destroy_result(result_handle);
                Ok(items)
            } else {
                Err(InventoryError::GetResultItemsFailed)
            }
        }
    }

    /// Destroy a result handle after use.
    pub fn destroy_result(&self, result_handle: sys::SteamInventoryResult_t) {
        unsafe {
            sys::SteamAPI_ISteamInventory_DestroyResult(
                self.inventory,
                result_handle,
            );
        }
    }
}

/// Represents an individual inventory item with its unique details.
#[derive(Clone, Debug)]
pub struct SteamItemDetails {
    pub item_id: SteamItemInstanceID,
    pub definition: SteamItemDef,
    pub quantity: u16,
    pub flags: u16,
}

/// Represents a unique identifier for an inventory item instance.
#[derive(Clone, Debug)]
pub struct SteamItemInstanceID(pub u64);

/// Represents a unique identifier for an item definition.
#[derive(Clone, Debug)]
pub struct SteamItemDef(pub i32);

/// Enumerates possible errors that can occur during inventory operations.
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

    #[test]
    fn test_get_result_items() {
        let client = Client::init().unwrap();
        let callback_processed = Arc::new(Mutex::new(false));
        let processed_clone = callback_processed.clone();

        client.register_callback(move |val: SteamInventoryResultReady| {
            assert!(val.result.is_ok(), "SteamInventoryResultReady Failed.");
            let result_items = Client::init().unwrap().inventory().get_result_items(val.handle);
            assert!(result_items.is_ok(), "Failed to get result items: {:?}", result_items.err().unwrap());
            println!("Result items: {:?}", result_items.unwrap());

            let mut processed = processed_clone.lock().unwrap();
            *processed = true;
        });

        client.register_callback(move |val: SteamInventoryFullUpdate| {
            println!("SteamInventoryFullUpdate: {:?}", val)
        });

        let _result = client.inventory().get_all_items();

        for _ in 0..50 {
            client.run_callbacks();
            let processed = callback_processed.lock().unwrap();
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
            if *processed {
                return;
            }
        }
        panic!("Timed out waiting for inventory result.");
    }
}