// "Tifflin" Kernel
// - By John Hodge (thePowersGang)
//
// Modules/syscalls/threads.rs
//! Thread management calls

use kernel::prelude::*;

use ObjectHandle;
use Error;
use values;
use args::Args;
//use kernel::threads::get_process_local;

/// Current process type (provides an object handle for IPC)
pub struct CurProcess;
impl ::objects::Object for CurProcess
{
	const CLASS: u16 = values::CLASS_CORE_THISPROCESS;
	fn class(&self) -> u16 { Self::CLASS }
	fn as_any(&self) -> &Any { self }
	fn handle_syscall_ref(&self, call: u16, args: &mut Args) -> Result<u64, Error>
	{
		match call
		{
		values::CORE_THISPROCESS_RECVOBJ => {
			let class: u16 = try!(args.get());
			Ok( ::objects::get_unclaimed(class) )
			},
		_ => ::objects::object_has_no_such_method_ref("threads::CurProcess", call),
		}
	}
	//fn handle_syscall_val(self, call: u16, _args: &mut Args) -> Result<u64,Error> {
	//	::objects::object_has_no_such_method_val("threads::CurProcess", call)
	//}
	fn bind_wait(&self, _flags: u32, _obj: &mut ::kernel::threads::SleepObject) -> u32 {
		0
	}
	fn clear_wait(&self, _flags: u32, _obj: &mut ::kernel::threads::SleepObject) -> u32 {
		0
	}
}

#[inline(never)]
pub fn exit(status: u32) {
	::kernel::threads::exit_process(status);
}
#[inline(never)]
pub fn terminate() {
	todo!("terminate()");
}
#[inline(never)]
pub fn newthread(sp: usize, ip: usize) -> ObjectHandle {
	todo!("newthread(sp={:#x},ip={:#x})", sp, ip);
}
#[inline(never)]
pub fn newprocess(name: &str,  clone_start: usize, clone_end: usize) -> ObjectHandle {
	// 1. Create a new process image (virtual address space)
	let process = ::kernel::threads::ProcessHandle::new(name, clone_start, clone_end);
	
	::objects::new_object( ProtoProcess(process) )
}

// ret: number of events triggered
#[inline(never)]
pub fn wait(events: &mut [values::WaitItem], wake_time_mono: u64) -> Result<u32,Error>
{
	let mut waiter = ::kernel::threads::SleepObject::new("wait");
	let mut num_bound = 0;
	for ev in events.iter() {
		num_bound += try!(::objects::wait_on_object(ev.object, ev.flags, &mut waiter));
	}

	if num_bound == 0 && wake_time_mono == !0 {
		// Attempting to sleep on no events with an infinite timeout! Would sleep forever
		log_error!("TODO: What to do when a thread tries to sleep forever");
		waiter.wait();
	}

	// A wake time of 0 means to not sleep at all, just check the status of the events
	// TODO: There should be a more efficient way of doing this, than binding only to unbind again
	if wake_time_mono != 0 {
		// !0 indicates an unbounded wait (no need to set a wakeup time)
		if wake_time_mono != !0 {
			todo!("Set a wakeup timer at {}", wake_time_mono);
			//waiter.wait_until(wake_time_mono);
		}
		else {
			waiter.wait();
		}
	}

	Ok( events.iter_mut().fold(0, |total,ev| total + ::objects::clear_wait(ev.object, ev.flags, &mut waiter).unwrap()) )
}

pub struct ProtoProcess(::kernel::threads::ProcessHandle);
impl ::objects::Object for ProtoProcess
{
	const CLASS: u16 = values::CLASS_CORE_PROTOPROCESS;
	fn class(&self) -> u16 { Self::CLASS }
	fn as_any(&self) -> &Any { self }
	fn handle_syscall_ref(&self, call: u16, args: &mut Args) -> Result<u64,Error>
	{
		match call
		{
		// Request termination of child process
		values::CORE_PROTOPROCESS_SENDOBJ => {
			let handle: u32 = try!(args.get());
			::objects::give_object(&self.0, handle).map(|_| 0)
			}
		_ => ::objects::object_has_no_such_method_ref("threads::ProtoProcess", call),
		}
	}
	fn handle_syscall_val(&mut self, call: u16, args: &mut Args) -> Result<u64,Error> {
		match call
		{
		values::CORE_PROTOPROCESS_START => {
			let ip: usize = try!(args.get());
			let sp: usize = try!(args.get());
			
			// HACK: Leaves the value as "valid" (but dropped)
			// SAFE: Raw pointer coerced from &mut
			let mut inner = unsafe { ::core::ptr::read_and_drop(&mut self.0) };

			// NOTE: Don't need to validate these values, as they're used only in user-space
			inner.start_root_thread(ip, sp);
			Ok( ::objects::new_object( Process(inner) ) as u64 )
			},
		_ => ::objects::object_has_no_such_method_val("threads::ProtoProcess", call)
		}
	}
	fn bind_wait(&self, _flags: u32, _obj: &mut ::kernel::threads::SleepObject) -> u32 {
		0
	}
	fn clear_wait(&self, _flags: u32, _obj: &mut ::kernel::threads::SleepObject) -> u32 {
		0
	}
}

pub struct Process(::kernel::threads::ProcessHandle);
impl ::objects::Object for Process
{
	const CLASS: u16 = values::CLASS_CORE_PROCESS;
	fn class(&self) -> u16 { Self::CLASS }
	fn as_any(&self) -> &Any { self }
	fn handle_syscall_ref(&self, call: u16, _args: &mut Args) -> Result<u64,Error>
	{
		match call
		{
		// Request termination of child process
		values::CORE_PROCESS_KILL => todo!("CORE_PROCESS_KILL"),
		_ => ::objects::object_has_no_such_method_ref("threads::Process", call),
		}
	}
	//fn handle_syscall_val(self, call: u16, _args: &mut Args) -> Result<u64,Error> {
	//	::objects::object_has_no_such_method_val("threads::process", call)
	//}
	fn bind_wait(&self, flags: u32, obj: &mut ::kernel::threads::SleepObject) -> u32 {
		let mut ret = 0;
		// Wait for child process to terminate
		if flags & values::EV_PROCESS_TERMINATED != 0 {
			self.0.bind_wait_terminate(obj);
			ret += 1;
		}
		ret
	}
	fn clear_wait(&self, flags: u32, obj: &mut ::kernel::threads::SleepObject) -> u32 {
		let mut ret = 0;
		// Wait for child process to terminate
		if flags & values::EV_PROCESS_TERMINATED != 0 {
			if self.0.clear_wait_terminate(obj) {
				ret |= values::EV_PROCESS_TERMINATED;
			}
		}
		ret
	}
}
