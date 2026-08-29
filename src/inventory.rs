//! Access to the Steam Inventory Service (`ISteamInventory`).
//!
//! Every inventory request that talks to Steam is asynchronous: the call
//! returns a result handle immediately and the result becomes ready later,
//! signalled through the `SteamInventoryResultReady_t` callback. This module
//! hides the handle plumbing behind one method per request, each of which
//! takes a closure that is invoked from [`Client::run_callbacks`] with either
//! an [`InventoryResult`] (whose handle is destroyed on drop) or the
//! [`SteamError`] Steam reported for the request.
//!
//! Because the crate can only hold one handler per callback id, the
//! `SteamInventoryResultReady_t` callback is owned by this module and is not
//! exposed for manual registration; [`InventoryFullUpdate`] and
//! [`InventoryDefinitionUpdate`] are.

use super::*;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicBool, Ordering};

/// An item definition id, as authored in the game's itemdefs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ItemDefId(pub i32);

/// The id of one item instance in a user's inventory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ItemInstanceId(pub u64);

impl ItemInstanceId {
    /// The sentinel Steam uses for "no item" (`k_SteamItemInstanceIDInvalid`).
    pub const INVALID: ItemInstanceId = ItemInstanceId(u64::MAX);
}

/// Per-item flags reported with an [`ItemDetails`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ItemFlags(pub u16);

impl ItemFlags {
    /// The item cannot be traded or sold on the market.
    pub const NO_TRADE: ItemFlags = ItemFlags(sys::ESteamItemFlags::k_ESteamItemNoTrade.0 as u16);
    /// The item was removed from the inventory by this result.
    pub const REMOVED: ItemFlags = ItemFlags(sys::ESteamItemFlags::k_ESteamItemRemoved.0 as u16);
    /// The item was consumed by this result.
    pub const CONSUMED: ItemFlags = ItemFlags(sys::ESteamItemFlags::k_ESteamItemConsumed.0 as u16);

    pub fn contains(self, other: ItemFlags) -> bool {
        self.0 & other.0 == other.0
    }
}

/// One item as reported by an [`InventoryResult`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ItemDetails {
    pub item_id: ItemInstanceId,
    pub definition: ItemDefId,
    pub quantity: u16,
    pub flags: ItemFlags,
}

impl From<sys::SteamItemDetails_t> for ItemDetails {
    fn from(raw: sys::SteamItemDetails_t) -> Self {
        ItemDetails {
            item_id: ItemInstanceId(raw.m_itemId),
            definition: ItemDefId(raw.m_iDefinition),
            quantity: raw.m_unQuantity,
            flags: ItemFlags(raw.m_unFlags),
        }
    }
}

/// A finished inventory result. The underlying Steam result handle is
/// destroyed when this is dropped.
pub struct InventoryResult {
    inventory: *mut sys::ISteamInventory,
    handle: sys::SteamInventoryResult_t,
    _inner: Arc<Inner>,
}

// The interface pointer is process-global and the handle is a plain integer;
// results may be read from any thread.
unsafe impl Send for InventoryResult {}
unsafe impl Sync for InventoryResult {}

impl InventoryResult {
    /// The raw result handle, for APIs this crate does not wrap.
    pub fn handle(&self) -> sys::SteamInventoryResult_t {
        self.handle
    }

    /// The status Steam reported for the request.
    pub fn status(&self) -> SteamResult {
        unsafe {
            to_steam_result(sys::SteamAPI_ISteamInventory_GetResultStatus(
                self.inventory,
                self.handle,
            ))
        }
    }

    /// The server time the result was generated, as a unix timestamp.
    pub fn timestamp(&self) -> u32 {
        unsafe { sys::SteamAPI_ISteamInventory_GetResultTimestamp(self.inventory, self.handle) }
    }

    /// Whether the result describes the inventory of the given user.
    pub fn check_steam_id(&self, steam_id: SteamId) -> bool {
        unsafe {
            sys::SteamAPI_ISteamInventory_CheckResultSteamID(
                self.inventory,
                self.handle,
                steam_id.0,
            )
        }
    }

    /// The items the result describes. Empty when the result is not ready or
    /// did not succeed.
    pub fn items(&self) -> Vec<ItemDetails> {
        unsafe {
            let mut count: u32 = 0;
            if !sys::SteamAPI_ISteamInventory_GetResultItems(
                self.inventory,
                self.handle,
                std::ptr::null_mut(),
                &mut count,
            ) {
                return Vec::new();
            }
            let mut raw = vec![
                sys::SteamItemDetails_t {
                    m_itemId: 0,
                    m_iDefinition: 0,
                    m_unQuantity: 0,
                    m_unFlags: 0,
                };
                count as usize
            ];
            if !sys::SteamAPI_ISteamInventory_GetResultItems(
                self.inventory,
                self.handle,
                raw.as_mut_ptr(),
                &mut count,
            ) {
                return Vec::new();
            }
            raw.truncate(count as usize);
            raw.into_iter().map(ItemDetails::from).collect()
        }
    }

    /// A dynamic property of the item at `index` in [`items`](Self::items).
    /// With `None` for the name, returns the comma-separated property names.
    pub fn item_property(&self, index: u32, name: Option<&str>) -> Option<String> {
        let name = name.map(|n| CString::new(n).ok()).unwrap_or(None);
        let name_ptr = name.as_ref().map_or(std::ptr::null(), |n| n.as_ptr());
        unsafe {
            read_string(|buf, len| {
                sys::SteamAPI_ISteamInventory_GetResultItemProperty(
                    self.inventory,
                    self.handle,
                    index,
                    name_ptr,
                    buf,
                    len,
                )
            })
        }
    }

    /// Serializes the result so a trusted third party can verify it with
    /// `ISteamInventory::DeserializeResult`.
    pub fn serialize(&self) -> Option<Vec<u8>> {
        unsafe {
            let mut len: u32 = 0;
            if !sys::SteamAPI_ISteamInventory_SerializeResult(
                self.inventory,
                self.handle,
                std::ptr::null_mut(),
                &mut len,
            ) {
                return None;
            }
            let mut buf = vec![0u8; len as usize];
            if !sys::SteamAPI_ISteamInventory_SerializeResult(
                self.inventory,
                self.handle,
                buf.as_mut_ptr() as *mut _,
                &mut len,
            ) {
                return None;
            }
            buf.truncate(len as usize);
            Some(buf)
        }
    }
}

impl Drop for InventoryResult {
    fn drop(&mut self) {
        unsafe {
            sys::SteamAPI_ISteamInventory_DestroyResult(self.inventory, self.handle);
        }
    }
}

/// Reads a string through Steam's "call once for the size, once for the
/// bytes" convention. The size reported by the first call includes the NUL.
unsafe fn read_string(
    mut call: impl FnMut(*mut std::os::raw::c_char, *mut u32) -> bool,
) -> Option<String> {
    let mut len: u32 = 0;
    if !call(std::ptr::null_mut(), &mut len) {
        return None;
    }
    let mut buf = vec![0u8; len as usize];
    if !call(buf.as_mut_ptr() as *mut _, &mut len) {
        return None;
    }
    let s = CStr::from_bytes_until_nul(&buf).ok()?;
    Some(s.to_string_lossy().into_owned())
}

type PendingCallback = Box<dyn FnOnce(Result<InventoryResult, SteamError>) + Send + 'static>;

/// Requests whose results have not been reported yet, keyed by result handle.
/// Lives in [`Inner`] so the dispatcher is registered once per client.
#[derive(Default)]
pub(crate) struct PendingResults {
    registered: AtomicBool,
    map: Mutex<HashMap<sys::SteamInventoryResult_t, PendingCallback>>,
}

struct InventoryResultReady {
    handle: sys::SteamInventoryResult_t,
    result: sys::EResult,
}

impl_callback!(cb: SteamInventoryResultReady_t => InventoryResultReady {
    Self {
        handle: cb.m_handle,
        result: cb.m_result,
    }
});

/// Fired when a full inventory update arrives for the local user (for
/// instance after a [`get_all_items`](Inventory::get_all_items) that found
/// changes). The handle is the same one reported to that request's closure.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InventoryFullUpdate {
    pub handle: sys::SteamInventoryResult_t,
}

impl_callback!(cb: SteamInventoryFullUpdate_t => InventoryFullUpdate {
    Self { handle: cb.m_handle }
});

/// Fired when item definitions have been loaded or updated, after
/// [`load_item_definitions`](Inventory::load_item_definitions).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InventoryDefinitionUpdate;

impl_callback!(_cb: SteamInventoryDefinitionUpdate_t => InventoryDefinitionUpdate {
    Self
});

/// Access to the steam inventory interface.
pub struct Inventory {
    pub(crate) inventory: *mut sys::ISteamInventory,
    pub(crate) inner: Arc<Inner>,
}

impl Inventory {
    pub(crate) fn new(inventory: *mut sys::ISteamInventory, inner: Arc<Inner>) -> Self {
        let this = Inventory { inventory, inner };
        this.ensure_dispatcher();
        this
    }

    /// Registers the shared `SteamInventoryResultReady_t` handler once per
    /// client. The handle is forgotten on purpose: the handler must outlive
    /// every `Inventory` accessor and is dropped with the client's callback map.
    fn ensure_dispatcher(&self) {
        let pending = &self.inner.inventory_results;
        if pending
            .registered
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        let inventory = self.inventory as usize;
        let weak = Arc::downgrade(&self.inner);
        let handle = unsafe {
            register_callback::<InventoryResultReady, _>(&self.inner, move |ready| {
                let Some(inner) = weak.upgrade() else { return };
                let cb = inner
                    .inventory_results
                    .map
                    .lock()
                    .unwrap()
                    .remove(&ready.handle);
                let Some(cb) = cb else { return };
                let result = InventoryResult {
                    inventory: inventory as *mut sys::ISteamInventory,
                    handle: ready.handle,
                    _inner: inner,
                };
                match to_steam_result(ready.result) {
                    Ok(()) => cb(Ok(result)),
                    Err(err) => {
                        drop(result);
                        cb(Err(err));
                    }
                }
            })
        };
        std::mem::forget(handle);
    }

    /// Issues a request and parks its closure until the result is ready. The
    /// pending map is held across the call so a result reported from another
    /// thread cannot be dispatched before the closure is registered.
    fn submit<F>(&self, call: impl FnOnce(*mut sys::SteamInventoryResult_t) -> bool, cb: F)
    where
        F: FnOnce(Result<InventoryResult, SteamError>) + Send + 'static,
    {
        let mut pending = self.inner.inventory_results.map.lock().unwrap();
        let mut handle = sys::k_SteamInventoryResultInvalid;
        let ok = call(&mut handle);
        if !ok || handle == sys::k_SteamInventoryResultInvalid {
            drop(pending);
            cb(Err(SteamError::Generic));
            return;
        }
        pending.insert(handle, Box::new(cb));
    }

    /// Requests the current user's entire inventory.
    pub fn get_all_items<F>(&self, cb: F)
    where
        F: FnOnce(Result<InventoryResult, SteamError>) + Send + 'static,
    {
        self.submit(
            |handle| unsafe { sys::SteamAPI_ISteamInventory_GetAllItems(self.inventory, handle) },
            cb,
        )
    }

    /// Requests the given item instances.
    pub fn get_items_by_id<F>(&self, ids: &[ItemInstanceId], cb: F)
    where
        F: FnOnce(Result<InventoryResult, SteamError>) + Send + 'static,
    {
        let raw: Vec<u64> = ids.iter().map(|id| id.0).collect();
        self.submit(
            |handle| unsafe {
                sys::SteamAPI_ISteamInventory_GetItemsByID(
                    self.inventory,
                    handle,
                    raw.as_ptr(),
                    raw.len() as u32,
                )
            },
            cb,
        )
    }

    /// Consumes `quantity` of an item instance. Only works for items whose
    /// definition allows it.
    pub fn consume_item<F>(&self, item: ItemInstanceId, quantity: u32, cb: F)
    where
        F: FnOnce(Result<InventoryResult, SteamError>) + Send + 'static,
    {
        self.submit(
            |handle| unsafe {
                sys::SteamAPI_ISteamInventory_ConsumeItem(self.inventory, handle, item.0, quantity)
            },
            cb,
        )
    }

    /// Crafts: destroys the given quantities of the `destroy` instances and
    /// generates the `generate` definitions, as one atomic exchange that must
    /// match an `exchange` recipe in the itemdefs.
    pub fn exchange_items<F>(
        &self,
        generate: &[(ItemDefId, u32)],
        destroy: &[(ItemInstanceId, u32)],
        cb: F,
    ) where
        F: FnOnce(Result<InventoryResult, SteamError>) + Send + 'static,
    {
        let gen_defs: Vec<i32> = generate.iter().map(|(d, _)| d.0).collect();
        let gen_qty: Vec<u32> = generate.iter().map(|(_, q)| *q).collect();
        let del_ids: Vec<u64> = destroy.iter().map(|(i, _)| i.0).collect();
        let del_qty: Vec<u32> = destroy.iter().map(|(_, q)| *q).collect();
        self.submit(
            |handle| unsafe {
                sys::SteamAPI_ISteamInventory_ExchangeItems(
                    self.inventory,
                    handle,
                    gen_defs.as_ptr(),
                    gen_qty.as_ptr(),
                    gen_defs.len() as u32,
                    del_ids.as_ptr(),
                    del_qty.as_ptr(),
                    del_ids.len() as u32,
                )
            },
            cb,
        )
    }

    /// Grants items directly. Only available to developers of the app.
    pub fn generate_items<F>(&self, items: &[(ItemDefId, u32)], cb: F)
    where
        F: FnOnce(Result<InventoryResult, SteamError>) + Send + 'static,
    {
        let defs: Vec<i32> = items.iter().map(|(d, _)| d.0).collect();
        let qty: Vec<u32> = items.iter().map(|(_, q)| *q).collect();
        self.submit(
            |handle| unsafe {
                sys::SteamAPI_ISteamInventory_GenerateItems(
                    self.inventory,
                    handle,
                    defs.as_ptr(),
                    qty.as_ptr(),
                    defs.len() as u32,
                )
            },
            cb,
        )
    }

    /// Moves `quantity` from one stack to another, or to a new stack when
    /// `dest` is `None`.
    pub fn transfer_item_quantity<F>(
        &self,
        source: ItemInstanceId,
        quantity: u32,
        dest: Option<ItemInstanceId>,
        cb: F,
    ) where
        F: FnOnce(Result<InventoryResult, SteamError>) + Send + 'static,
    {
        let dest = dest.unwrap_or(ItemInstanceId::INVALID);
        self.submit(
            |handle| unsafe {
                sys::SteamAPI_ISteamInventory_TransferItemQuantity(
                    self.inventory,
                    handle,
                    source.0,
                    quantity,
                    dest.0,
                )
            },
            cb,
        )
    }

    /// Asks Steam to roll a playtime drop against the given generator
    /// definition.
    pub fn trigger_item_drop<F>(&self, drop_list: ItemDefId, cb: F)
    where
        F: FnOnce(Result<InventoryResult, SteamError>) + Send + 'static,
    {
        self.submit(
            |handle| unsafe {
                sys::SteamAPI_ISteamInventory_TriggerItemDrop(self.inventory, handle, drop_list.0)
            },
            cb,
        )
    }

    /// Grants every promo item the user is eligible for.
    pub fn grant_promo_items<F>(&self, cb: F)
    where
        F: FnOnce(Result<InventoryResult, SteamError>) + Send + 'static,
    {
        self.submit(
            |handle| unsafe {
                sys::SteamAPI_ISteamInventory_GrantPromoItems(self.inventory, handle)
            },
            cb,
        )
    }

    /// Grants the given promo items, if the user is eligible.
    pub fn add_promo_items<F>(&self, defs: &[ItemDefId], cb: F)
    where
        F: FnOnce(Result<InventoryResult, SteamError>) + Send + 'static,
    {
        let raw: Vec<i32> = defs.iter().map(|d| d.0).collect();
        self.submit(
            |handle| unsafe {
                sys::SteamAPI_ISteamInventory_AddPromoItems(
                    self.inventory,
                    handle,
                    raw.as_ptr(),
                    raw.len() as u32,
                )
            },
            cb,
        )
    }

    /// Tells Steam the user is playing, for playtime-based drops.
    pub fn send_item_drop_heartbeat(&self) {
        unsafe { sys::SteamAPI_ISteamInventory_SendItemDropHeartbeat(self.inventory) }
    }

    /// Starts loading the item definitions; [`InventoryDefinitionUpdate`]
    /// fires when they are available.
    pub fn load_item_definitions(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamInventory_LoadItemDefinitions(self.inventory) }
    }

    /// The ids of every loaded item definition, or `None` when definitions
    /// have not been loaded yet.
    pub fn item_definition_ids(&self) -> Option<Vec<ItemDefId>> {
        unsafe {
            let mut count: u32 = 0;
            if !sys::SteamAPI_ISteamInventory_GetItemDefinitionIDs(
                self.inventory,
                std::ptr::null_mut(),
                &mut count,
            ) {
                return None;
            }
            let mut raw = vec![0i32; count as usize];
            if !sys::SteamAPI_ISteamInventory_GetItemDefinitionIDs(
                self.inventory,
                raw.as_mut_ptr(),
                &mut count,
            ) {
                return None;
            }
            raw.truncate(count as usize);
            Some(raw.into_iter().map(ItemDefId).collect())
        }
    }

    /// A property of a loaded item definition. With `None` for the name,
    /// returns the comma-separated property names.
    pub fn item_definition_property(&self, def: ItemDefId, name: Option<&str>) -> Option<String> {
        let name = name.map(|n| CString::new(n).ok()).unwrap_or(None);
        let name_ptr = name.as_ref().map_or(std::ptr::null(), |n| n.as_ptr());
        unsafe {
            read_string(|buf, len| {
                sys::SteamAPI_ISteamInventory_GetItemDefinitionProperty(
                    self.inventory,
                    def.0,
                    name_ptr,
                    buf,
                    len,
                )
            })
        }
    }
}
