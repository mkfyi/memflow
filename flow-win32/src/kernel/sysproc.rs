use super::StartBlock;
use crate::error::{Error, Result};
use crate::pe::{self, MemoryPeViewContext};

use log::{debug, info, warn};

use flow_core::mem::VirtualMemory;
use flow_core::types::{Address, Length};

use byteorder::{ByteOrder, LittleEndian};

use pelite::{
    self,
    pe64::exports::{Export, GetProcAddress},
};

pub fn find<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    debug!("trying to find system eprocess");

    match find_exported(virt_mem, start_block, ntos) {
        Ok(e) => return Ok(e),
        Err(e) => warn!("{}", e),
    }

    match find_in_section(virt_mem, start_block, ntos) {
        Ok(e) => return Ok(e),
        Err(e) => warn!("{}", e),
    }

    Err(Error::new("unable to find system eprocess"))
}

// find from exported symbol
pub fn find_exported<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    start_block: &StartBlock,
    kernel_base: Address,
) -> Result<Address> {
    // PsInitialSystemProcess -> PsActiveProcessHead
    let ctx = MemoryPeViewContext::new(virt_mem, kernel_base)?;
    let pe = pe::wrap_memory_pe_view(&ctx)?;
    let sys_proc = match pe.get_export_by_name("PsInitialSystemProcess")? {
        Export::Symbol(s) => kernel_base + Length::from(*s),
        Export::Forward(_) => {
            return Err(Error::new(
                "PsInitialSystemProcess found but it was a forwarded export",
            ))
        }
    };
    info!("PsInitialSystemProcess found at 0x{:x}", sys_proc);

    // read containing value
    let mut buf = vec![0u8; start_block.arch.len_addr().as_usize()];
    let sys_proc_addr: Address = match start_block.arch.bits() {
        64 => {
            virt_mem.virt_read_raw_into(sys_proc, &mut buf)?;
            LittleEndian::read_u64(&buf).into()
        }
        32 => {
            virt_mem.virt_read_raw_into(sys_proc, &mut buf)?;
            LittleEndian::read_u32(&buf).into()
        }
        _ => return Err(Error::new("invalid architecture")),
    };
    Ok(sys_proc_addr)
}

// scan in pdb

// scan in section
pub fn find_in_section<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    _start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    // find section ALMOSTRO
    // scan for va of system process (dtb.va)
    // ... check if its 32 or 64bit

    let mut header_buf = vec![0; Length::from_mb(32).as_usize()];
    virt_mem.virt_read_raw_into(ntos, &mut header_buf)?;

    /*
    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let header = PE::parse_with_opts(&header_buf, &pe_opts).unwrap(); // TODO: error
    let _sect = header
        .sections
        .iter()
        .filter(|s| String::from_utf8(s.name.to_vec()).unwrap_or_default() == "ALMOSTRO")
        .nth(0)
        .ok_or_else(|| Error::new("unable to find section ALMOSTRO"))?;
    */

    Err(Error::new(
        "sysproc::find_in_section(): not implemented yet",
    ))
}
