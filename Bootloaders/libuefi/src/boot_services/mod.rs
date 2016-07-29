
use super::{Void,Status,Guid,Handle,Event};
use super::{PhysicalAddress,VirtualAddress};

pub mod protocols;

/// Task Priority Level
pub type Tpl = usize;

//#[repr(C)]
pub type PoolPointer<T> = *mut T;


#[repr(C)]
pub struct BootServices
{
	pub hdr: super::TableHeader,
	
	// Task Priority
	pub raise_tpl: extern "win64" fn(Tpl) -> Tpl,
	pub restore_tpl: extern "win64" fn(Tpl) -> Tpl,

	// Memory
	pub allocate_pages: extern "win64" fn(AllocateType, MemoryType, no_pages: usize, phys_addr: &mut PhysicalAddress) -> Status,
	pub free_pages: extern "win64" fn(PhysicalAddress, usize) -> Status,
	pub get_memory_map: extern "win64" fn(&mut usize, *mut MemoryDescriptor, map_key: &mut usize, descriptor_size: &mut usize, descriptor_version: &mut u32) -> Status,
	pub allocate_pool: extern "win64" fn(MemoryType, usize, &mut PoolPointer<Void>) -> Status,
	pub free_pool: extern "win64" fn(*mut Void) -> Status,

	// Timing and events
	pub create_event: extern "win64" fn(u32, notify_tpl: Tpl, notify_function: EventNotifyFcn, *mut Void, &mut Event) -> Status,
	pub set_timer: extern "win64" fn(Event, TimerDelay, u64) -> Status,
	pub wait_for_event: extern "win64" fn(usize, *mut Event, &mut usize) -> Status,
	pub signal_event: extern "win64" fn(Event) -> Status,
	pub close_event: extern "win64" fn(Event) -> Status,
	pub check_event: extern "win64" fn(Event) -> Status,

	// Protocol handler functions
	pub install_protocol_interface: extern "win64" fn(&mut Handle, &Guid, InterfaceType, *mut Void) -> Status,
	pub reinstall_protocol_interface: extern "win64" fn(Handle, &Guid, old: *mut Void, new: *mut Void) -> Status,
	pub uninstall_protocol_interface: extern "win64" fn(Handle, &Guid, *mut Void) -> Status,
	pub handle_protocol: extern "win64" fn(Handle, &Guid, &mut *mut Void) -> Status,
	pub pc_handle_protocol: extern "win64" fn(Handle, &Guid, &mut *mut Void) -> Status,
	pub register_protocol_notify: extern "win64" fn(&Guid, Event, &mut *mut Void) -> Status,
	pub locate_handle: extern "win64" fn(LocateSearchType, Option<&Guid>, *mut Void, &mut usize, *mut Handle) -> Status,
	pub locate_device_path: extern "win64" fn(&Guid, &mut *mut DevicePath, &mut Handle) -> Status,
	pub install_configuration_table: extern "win64" fn(&Guid, *mut Void) -> Status,

	// Image functions
	pub load_image: extern "win64" fn(bool, Handle, &DevicePath, *mut Void, usize, &mut Handle) -> Status,
	pub start_image: extern "win64" fn(Handle, &mut usize, Option<&mut *mut u16>) -> Status,
	pub exit: extern "win64" fn(Handle, Status, usize, *const u16) -> Status,
	pub unload_image: extern "win64" fn(Handle) -> Status,
	pub exit_boot_services: extern "win64" fn(Handle, map_key: usize) -> Status,
	
	// Misc functions
	pub get_next_monotonic_count: extern "win64" fn() -> Status,
	pub stall: extern "win64" fn() -> Status,
	pub set_watchdog_timer: extern "win64" fn() -> Status,

	// DriverSupport Services
	pub connect_controller: extern "win64" fn() -> Status,
	pub disconnect_controller: extern "win64" fn() -> Status,

	// Open/Close Protocol Services
	pub open_protocol: extern "win64" fn(Handle, &Guid, Option<&mut *mut Void>, Handle, Handle, u32) -> Status,
	pub close_protocol: extern "win64" fn(Handle, &Guid, Handle, Handle) -> Status,
	pub open_protocol_information: extern "win64" fn() -> Status,

	// Library Services
	pub protocols_per_handle: extern "win64" fn(Handle, &mut PoolPointer<&Guid>, &mut usize) -> Status,
	pub locate_handle_buffer: extern "win64" fn(LocateSearchType, Option<&Guid>, *const Void, &mut usize, &mut *mut Handle) -> Status,
	pub locate_protocol: extern "win64" fn() -> Status,
	pub install_multiple_protocol_interfaces: extern "win64" fn() -> Status,
	pub uninstall_multiple_protocol_interfaces: extern "win64" fn() -> Status,

	// CRC
	pub calculate_crc32: extern "win64" fn() -> Status,

	// Misc Services
	pub copy_mem: extern "win64" fn() -> Status,
	pub set_mem: extern "win64" fn() -> Status,
	pub create_event_ex: extern "win64" fn() -> Status,
}
impl BootServices
{
	//pub fn locate_handles_by_protocol(&self, protocol: &Guid) -> Result<PoolSlice<Handle>, Status> {
	//	let mut ptr = 0 as *mut _;
	//	let mut count = 0;
	//	(self.locate_handle_buffer)(LocateSearchType::ByProtocol, Some(protocol), 0 as *const _, &mut count, &mut ptr)
	//		.err_or_else(|| PoolSlice(ptr, count) )
	//}
	
	pub fn handle_protocol<'a, P: 'a + protocols::Protocol>(&'a self, handle: &Handle) -> Result<&'a P, Status> {
		let mut ptr = 0 as *mut Void;
		(self.handle_protocol)(*handle, &P::guid(), &mut ptr)
			.err_or_else( || unsafe { &*P::from_ptr(ptr) } )
	}
}

#[repr(C)]
pub struct MemoryDescriptor
{
	pub ty: u32,
	_pad: u32,
	pub physical_start: PhysicalAddress,
	pub virtual_start: VirtualAddress,
	pub number_of_pages: u64,
	pub attribute: u64,
}
#[repr(C)]
pub enum MemoryType
{
    ReservedMemoryType,
    LoaderCode,
    LoaderData,
    BootServicesCode,
    BootServicesData,
    RuntimeServicesCode,
    RuntimeServicesData,
    ConventionalMemory,
    UnusableMemory,
    AcpiReclaimMemory,
    AcpiMemoryNVS,
    MemoryMappedIO,
    MemoryMappedIOPortSpace,
    PalCode,
    MaxMemoryType
}

#[repr(C)]
pub enum TimerDelay
{
	Cancel,
	Periodic,
	Relative,
}
#[repr(C)]
pub enum AllocateType
{
	AnyPages,
	MaxAddress,
	Address,
}
#[repr(C)]
pub enum InterfaceType
{
	Native,
	PCode,
}
#[repr(C)]
pub enum LocateSearchType
{
	AllHandles,
	ByRegisterNotify,
	ByProtocol,
}
#[repr(C)]
pub struct DevicePath
{
	ty: u8,
	sub_type: u8,
	length: [u8; 2],
}

pub type EventNotifyFcn = extern "win64" fn(Event, *mut Void);

