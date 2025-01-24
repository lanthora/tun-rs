#![allow(warnings)]
/* automatically generated by rust-bindgen 0.59.1 */
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct __BindgenBitfieldUnit<Storage> {
    storage: Storage,
}
impl<Storage> __BindgenBitfieldUnit<Storage> {
    #[inline]
    pub const fn new(storage: Storage) -> Self {
        Self { storage }
    }
}
impl<Storage> __BindgenBitfieldUnit<Storage>
where
    Storage: AsRef<[u8]> + AsMut<[u8]>,
{
    #[inline]
    pub fn get_bit(&self, index: usize) -> bool {
        debug_assert!(index / 8 < self.storage.as_ref().len());
        let byte_index = index / 8;
        let byte = self.storage.as_ref()[byte_index];
        let bit_index = if cfg!(target_endian = "big") {
            7 - (index % 8)
        } else {
            index % 8
        };
        let mask = 1 << bit_index;
        byte & mask == mask
    }
    #[inline]
    pub fn set_bit(&mut self, index: usize, val: bool) {
        debug_assert!(index / 8 < self.storage.as_ref().len());
        let byte_index = index / 8;
        let byte = &mut self.storage.as_mut()[byte_index];
        let bit_index = if cfg!(target_endian = "big") {
            7 - (index % 8)
        } else {
            index % 8
        };
        let mask = 1 << bit_index;
        if val {
            *byte |= mask;
        } else {
            *byte &= !mask;
        }
    }
    #[inline]
    pub fn get(&self, bit_offset: usize, bit_width: u8) -> u64 {
        debug_assert!(bit_width <= 64);
        debug_assert!(bit_offset / 8 < self.storage.as_ref().len());
        debug_assert!((bit_offset + (bit_width as usize)) / 8 <= self.storage.as_ref().len());
        let mut val = 0;
        for i in 0..(bit_width as usize) {
            if self.get_bit(i + bit_offset) {
                let index = if cfg!(target_endian = "big") {
                    bit_width as usize - 1 - i
                } else {
                    i
                };
                val |= 1 << index;
            }
        }
        val
    }
    #[inline]
    pub fn set(&mut self, bit_offset: usize, bit_width: u8, val: u64) {
        debug_assert!(bit_width <= 64);
        debug_assert!(bit_offset / 8 < self.storage.as_ref().len());
        debug_assert!((bit_offset + (bit_width as usize)) / 8 <= self.storage.as_ref().len());
        for i in 0..(bit_width as usize) {
            let mask = 1 << i;
            let val_bit_is_set = val & mask == mask;
            let index = if cfg!(target_endian = "big") {
                bit_width as usize - 1 - i
            } else {
                i
            };
            self.set_bit(index + bit_offset, val_bit_is_set);
        }
    }
}
pub type wchar_t = ::std::os::raw::c_ushort;
pub type DWORD = ::std::os::raw::c_ulong;
pub type BOOL = ::std::os::raw::c_int;
pub type BYTE = ::std::os::raw::c_uchar;
pub type ULONG64 = ::std::os::raw::c_ulonglong;
pub type DWORD64 = ::std::os::raw::c_ulonglong;
pub type WCHAR = wchar_t;
pub type LPCWSTR = *const WCHAR;
pub type HANDLE = *mut ::std::os::raw::c_void;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _GUID {
    pub Data1: ::std::os::raw::c_ulong,
    pub Data2: ::std::os::raw::c_ushort,
    pub Data3: ::std::os::raw::c_ushort,
    pub Data4: [::std::os::raw::c_uchar; 8usize],
}
#[test]
fn bindgen_test_layout__GUID() {
    assert_eq!(
        ::std::mem::size_of::<_GUID>(),
        16usize,
        concat!("Size of: ", stringify!(_GUID))
    );
    assert_eq!(
        ::std::mem::align_of::<_GUID>(),
        4usize,
        concat!("Alignment of ", stringify!(_GUID))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_GUID>())).Data1 as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(_GUID),
            "::",
            stringify!(Data1)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_GUID>())).Data2 as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(_GUID),
            "::",
            stringify!(Data2)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_GUID>())).Data3 as *const _ as usize },
        6usize,
        concat!(
            "Offset of field: ",
            stringify!(_GUID),
            "::",
            stringify!(Data3)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_GUID>())).Data4 as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(_GUID),
            "::",
            stringify!(Data4)
        )
    );
}
pub type GUID = _GUID;
#[repr(C)]
#[derive(Copy, Clone)]
pub union _NET_LUID_LH {
    pub Value: ULONG64,
    pub Info: _NET_LUID_LH__bindgen_ty_1,
}
#[repr(C)]
#[repr(align(8))]
#[derive(Debug, Copy, Clone)]
pub struct _NET_LUID_LH__bindgen_ty_1 {
    pub _bitfield_align_1: [u32; 0],
    pub _bitfield_1: __BindgenBitfieldUnit<[u8; 8usize]>,
}
#[test]
fn bindgen_test_layout__NET_LUID_LH__bindgen_ty_1() {
    assert_eq!(
        ::std::mem::size_of::<_NET_LUID_LH__bindgen_ty_1>(),
        8usize,
        concat!("Size of: ", stringify!(_NET_LUID_LH__bindgen_ty_1))
    );
    assert_eq!(
        ::std::mem::align_of::<_NET_LUID_LH__bindgen_ty_1>(),
        8usize,
        concat!("Alignment of ", stringify!(_NET_LUID_LH__bindgen_ty_1))
    );
}
impl _NET_LUID_LH__bindgen_ty_1 {
    #[inline]
    pub fn Reserved(&self) -> ULONG64 {
        unsafe { ::std::mem::transmute(self._bitfield_1.get(0usize, 24u8) as u64) }
    }
    #[inline]
    pub fn set_Reserved(&mut self, val: ULONG64) {
        unsafe {
            let val: u64 = ::std::mem::transmute(val);
            self._bitfield_1.set(0usize, 24u8, val as u64)
        }
    }
    #[inline]
    pub fn NetLuidIndex(&self) -> ULONG64 {
        unsafe { ::std::mem::transmute(self._bitfield_1.get(24usize, 24u8) as u64) }
    }
    #[inline]
    pub fn set_NetLuidIndex(&mut self, val: ULONG64) {
        unsafe {
            let val: u64 = ::std::mem::transmute(val);
            self._bitfield_1.set(24usize, 24u8, val as u64)
        }
    }
    #[inline]
    pub fn IfType(&self) -> ULONG64 {
        unsafe { ::std::mem::transmute(self._bitfield_1.get(48usize, 16u8) as u64) }
    }
    #[inline]
    pub fn set_IfType(&mut self, val: ULONG64) {
        unsafe {
            let val: u64 = ::std::mem::transmute(val);
            self._bitfield_1.set(48usize, 16u8, val as u64)
        }
    }
    #[inline]
    pub fn new_bitfield_1(
        Reserved: ULONG64,
        NetLuidIndex: ULONG64,
        IfType: ULONG64,
    ) -> __BindgenBitfieldUnit<[u8; 8usize]> {
        let mut __bindgen_bitfield_unit: __BindgenBitfieldUnit<[u8; 8usize]> = Default::default();
        __bindgen_bitfield_unit.set(0usize, 24u8, {
            let Reserved: u64 = unsafe { ::std::mem::transmute(Reserved) };
            Reserved as u64
        });
        __bindgen_bitfield_unit.set(24usize, 24u8, {
            let NetLuidIndex: u64 = unsafe { ::std::mem::transmute(NetLuidIndex) };
            NetLuidIndex as u64
        });
        __bindgen_bitfield_unit.set(48usize, 16u8, {
            let IfType: u64 = unsafe { ::std::mem::transmute(IfType) };
            IfType as u64
        });
        __bindgen_bitfield_unit
    }
}
#[test]
fn bindgen_test_layout__NET_LUID_LH() {
    assert_eq!(
        ::std::mem::size_of::<_NET_LUID_LH>(),
        8usize,
        concat!("Size of: ", stringify!(_NET_LUID_LH))
    );
    assert_eq!(
        ::std::mem::align_of::<_NET_LUID_LH>(),
        8usize,
        concat!("Alignment of ", stringify!(_NET_LUID_LH))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_NET_LUID_LH>())).Value as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(_NET_LUID_LH),
            "::",
            stringify!(Value)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_NET_LUID_LH>())).Info as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(_NET_LUID_LH),
            "::",
            stringify!(Info)
        )
    );
}
pub type NET_LUID_LH = _NET_LUID_LH;
pub type NET_LUID = NET_LUID_LH;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _WINTUN_ADAPTER {
    _unused: [u8; 0],
}
#[doc = " A handle representing Wintun adapter"]
pub type WINTUN_ADAPTER_HANDLE = *mut _WINTUN_ADAPTER;
#[doc = "< Informational"]
pub const WINTUN_LOGGER_LEVEL_WINTUN_LOG_INFO: WINTUN_LOGGER_LEVEL = 0;
#[doc = "< Warning"]
pub const WINTUN_LOGGER_LEVEL_WINTUN_LOG_WARN: WINTUN_LOGGER_LEVEL = 1;
#[doc = "< Error"]
pub const WINTUN_LOGGER_LEVEL_WINTUN_LOG_ERR: WINTUN_LOGGER_LEVEL = 2;
#[doc = " Determines the level of logging, passed to WINTUN_LOGGER_CALLBACK."]
pub type WINTUN_LOGGER_LEVEL = ::std::os::raw::c_int;
#[doc = " Called by internal logger to report diagnostic messages"]
#[doc = ""]
#[doc = " @param Level         Message level."]
#[doc = ""]
#[doc = " @param Timestamp     Message timestamp in in 100ns intervals since 1601-01-01 UTC."]
#[doc = ""]
#[doc = " @param Message       Message text."]
pub type WINTUN_LOGGER_CALLBACK = ::std::option::Option<
    unsafe extern "C" fn(Level: WINTUN_LOGGER_LEVEL, Timestamp: DWORD64, Message: LPCWSTR),
>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _TUN_SESSION {
    _unused: [u8; 0],
}
#[doc = " A handle representing Wintun session"]
pub type WINTUN_SESSION_HANDLE = *mut _TUN_SESSION;
extern crate libloading;
pub struct wintun {
    __library: ::libloading::Library,
    pub WintunCreateAdapter: unsafe extern "C" fn(
        arg1: LPCWSTR,
        arg2: LPCWSTR,
        arg3: *const GUID,
    ) -> WINTUN_ADAPTER_HANDLE,
    pub WintunCloseAdapter: unsafe extern "C" fn(arg1: WINTUN_ADAPTER_HANDLE),
    pub WintunOpenAdapter: unsafe extern "C" fn(arg1: LPCWSTR) -> WINTUN_ADAPTER_HANDLE,
    pub WintunGetAdapterLUID:
        unsafe extern "C" fn(arg1: WINTUN_ADAPTER_HANDLE, arg2: *mut NET_LUID),
    pub WintunGetRunningDriverVersion: unsafe extern "C" fn() -> DWORD,
    pub WintunDeleteDriver: unsafe extern "C" fn() -> BOOL,
    pub WintunSetLogger: unsafe extern "C" fn(arg1: WINTUN_LOGGER_CALLBACK),
    pub WintunStartSession:
        unsafe extern "C" fn(arg1: WINTUN_ADAPTER_HANDLE, arg2: DWORD) -> WINTUN_SESSION_HANDLE,
    pub WintunEndSession: unsafe extern "C" fn(arg1: WINTUN_SESSION_HANDLE),
    pub WintunGetReadWaitEvent: unsafe extern "C" fn(arg1: WINTUN_SESSION_HANDLE) -> HANDLE,
    pub WintunReceivePacket:
        unsafe extern "C" fn(arg1: WINTUN_SESSION_HANDLE, arg2: *mut DWORD) -> *mut BYTE,
    pub WintunReleaseReceivePacket:
        unsafe extern "C" fn(arg1: WINTUN_SESSION_HANDLE, arg2: *const BYTE),
    pub WintunAllocateSendPacket:
        unsafe extern "C" fn(arg1: WINTUN_SESSION_HANDLE, arg2: DWORD) -> *mut BYTE,
    pub WintunSendPacket: unsafe extern "C" fn(arg1: WINTUN_SESSION_HANDLE, arg2: *const BYTE),
}
impl wintun {
    pub unsafe fn new<P>(path: P) -> Result<Self, ::libloading::Error>
    where
        P: AsRef<::std::ffi::OsStr>,
    {
        let library = ::libloading::Library::new(path)?;
        Self::from_library(library)
    }
    pub unsafe fn from_library<L>(library: L) -> Result<Self, ::libloading::Error>
    where
        L: Into<::libloading::Library>,
    {
        let __library = library.into();
        let WintunCreateAdapter = __library.get(b"WintunCreateAdapter\0").map(|sym| *sym)?;
        let WintunCloseAdapter = __library.get(b"WintunCloseAdapter\0").map(|sym| *sym)?;
        let WintunOpenAdapter = __library.get(b"WintunOpenAdapter\0").map(|sym| *sym)?;
        let WintunGetAdapterLUID = __library.get(b"WintunGetAdapterLUID\0").map(|sym| *sym)?;
        let WintunGetRunningDriverVersion = __library
            .get(b"WintunGetRunningDriverVersion\0")
            .map(|sym| *sym)?;
        let WintunDeleteDriver = __library.get(b"WintunDeleteDriver\0").map(|sym| *sym)?;
        let WintunSetLogger = __library.get(b"WintunSetLogger\0").map(|sym| *sym)?;
        let WintunStartSession = __library.get(b"WintunStartSession\0").map(|sym| *sym)?;
        let WintunEndSession = __library.get(b"WintunEndSession\0").map(|sym| *sym)?;
        let WintunGetReadWaitEvent = __library.get(b"WintunGetReadWaitEvent\0").map(|sym| *sym)?;
        let WintunReceivePacket = __library.get(b"WintunReceivePacket\0").map(|sym| *sym)?;
        let WintunReleaseReceivePacket = __library
            .get(b"WintunReleaseReceivePacket\0")
            .map(|sym| *sym)?;
        let WintunAllocateSendPacket = __library
            .get(b"WintunAllocateSendPacket\0")
            .map(|sym| *sym)?;
        let WintunSendPacket = __library.get(b"WintunSendPacket\0").map(|sym| *sym)?;
        Ok(wintun {
            __library,
            WintunCreateAdapter,
            WintunCloseAdapter,
            WintunOpenAdapter,
            WintunGetAdapterLUID,
            WintunGetRunningDriverVersion,
            WintunDeleteDriver,
            WintunSetLogger,
            WintunStartSession,
            WintunEndSession,
            WintunGetReadWaitEvent,
            WintunReceivePacket,
            WintunReleaseReceivePacket,
            WintunAllocateSendPacket,
            WintunSendPacket,
        })
    }
    pub unsafe fn WintunCreateAdapter(
        &self,
        arg1: LPCWSTR,
        arg2: LPCWSTR,
        arg3: *const GUID,
    ) -> WINTUN_ADAPTER_HANDLE {
        (self.WintunCreateAdapter)(arg1, arg2, arg3)
    }
    pub unsafe fn WintunCloseAdapter(&self, arg1: WINTUN_ADAPTER_HANDLE) -> () {
        (self.WintunCloseAdapter)(arg1)
    }
    pub unsafe fn WintunOpenAdapter(&self, arg1: LPCWSTR) -> WINTUN_ADAPTER_HANDLE {
        (self.WintunOpenAdapter)(arg1)
    }
    pub unsafe fn WintunGetAdapterLUID(
        &self,
        arg1: WINTUN_ADAPTER_HANDLE,
        arg2: *mut NET_LUID,
    ) -> () {
        (self.WintunGetAdapterLUID)(arg1, arg2)
    }
    pub unsafe fn WintunGetRunningDriverVersion(&self) -> DWORD {
        (self.WintunGetRunningDriverVersion)()
    }
    pub unsafe fn WintunDeleteDriver(&self) -> BOOL {
        (self.WintunDeleteDriver)()
    }
    pub unsafe fn WintunSetLogger(&self, arg1: WINTUN_LOGGER_CALLBACK) -> () {
        (self.WintunSetLogger)(arg1)
    }
    pub unsafe fn WintunStartSession(
        &self,
        arg1: WINTUN_ADAPTER_HANDLE,
        arg2: DWORD,
    ) -> WINTUN_SESSION_HANDLE {
        (self.WintunStartSession)(arg1, arg2)
    }
    pub unsafe fn WintunEndSession(&self, arg1: WINTUN_SESSION_HANDLE) -> () {
        (self.WintunEndSession)(arg1)
    }
    pub unsafe fn WintunGetReadWaitEvent(&self, arg1: WINTUN_SESSION_HANDLE) -> HANDLE {
        (self.WintunGetReadWaitEvent)(arg1)
    }
    pub unsafe fn WintunReceivePacket(
        &self,
        arg1: WINTUN_SESSION_HANDLE,
        arg2: *mut DWORD,
    ) -> *mut BYTE {
        (self.WintunReceivePacket)(arg1, arg2)
    }
    pub unsafe fn WintunReleaseReceivePacket(
        &self,
        arg1: WINTUN_SESSION_HANDLE,
        arg2: *const BYTE,
    ) -> () {
        (self.WintunReleaseReceivePacket)(arg1, arg2)
    }
    pub unsafe fn WintunAllocateSendPacket(
        &self,
        arg1: WINTUN_SESSION_HANDLE,
        arg2: DWORD,
    ) -> *mut BYTE {
        (self.WintunAllocateSendPacket)(arg1, arg2)
    }
    pub unsafe fn WintunSendPacket(&self, arg1: WINTUN_SESSION_HANDLE, arg2: *const BYTE) -> () {
        (self.WintunSendPacket)(arg1, arg2)
    }
}
