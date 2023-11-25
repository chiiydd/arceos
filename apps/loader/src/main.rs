#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]
#[cfg(feature = "axstd")]
use axstd::println;
const PLASH_START: usize = 0x22000000;
const RUN_START: usize = 0x4010_0000;
const HEADER_LENGTH:usize=2;
const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE:usize = 3;
static  mut KERNERL_PAGE_ROOT:usize=0;

use riscv::register::satp;


//ABI TABLE 
static mut ABI_TABLE: [usize; 16] = [0; 16];
fn register_abi(num: usize, handle: usize) {
   unsafe { ABI_TABLE[num] = handle; }
}
fn abi_hello() {
   println!("[ABI:Hello] Hello, Apps!");
}
fn abi_putchar(c: char) {
   println!("[ABI:Print] {c}");
}
fn abi_terminate(){
   println!("[ABI:Terminate] ArceOS terminates.");
   axhal::misc::terminate();
}
 #[cfg_attr(feature = "axstd", no_mangle)]

 fn main() {
    // let header_start=PLASH_START as *const u8;
    // let apps_start: *const u8 = (PLASH_START+HEADER_LENGTH) as *const u8;
    // // let apps_size = 32; // Dangerous!!! We need to get accurate size of apps.
    // let apps_size_bytes:&[u8]=unsafe {core::slice::from_raw_parts(header_start, HEADER_LENGTH)};


    // let apps_size=apps_size_bytes[0] as usize;
   
    register_abi_table();

    println!("Load payload ...");



    load_and_run_apps();
    // let code = unsafe { core::slice::from_raw_parts(apps_start, apps_size) };
    // println!("app size:{:?} content:{:?}",apps_size,code);
    // println!("Load payload ok!");
 }


 // APP SPACE 

 #[link_section = ".data.app_page_table"]
static mut APP_PT_SV39: [[u64; 512];10] = [[0; 512];10];


unsafe fn init_app_page_table(app_amount:usize) {

   println!("init {} apps's page table",app_amount);
   for idx in 0..app_amount{
         // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
   APP_PT_SV39[idx][2] = (0x80000 << 10) | 0xef;
   // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
   APP_PT_SV39[idx][0x102] = (0x80000 << 10) | 0xef;
   // 0x0000_0000..0x4000_0000, VRWX_GAD, 1G block
   APP_PT_SV39[idx][0] = (0x00000 << 10) | 0xef;
   // For App aspace!
   // 0x4000_0000..0x8000_0000, VRWX_GAD, 1G block
   APP_PT_SV39[idx][1] = (0x80000 << 10) | 0xef;
   }
 
}
unsafe fn switch_to_physical_page() {

   
   // Configure satp to use Sv39 mode with physical page table address
   satp::set(satp::Mode::Sv39, 0, KERNERL_PAGE_ROOT >> 12);

   // Flush the TLB to ensure the new translation takes effect
   riscv::asm::sfence_vma_all();
}
unsafe fn switch_app_aspace(app_index:usize) {
   
   println!("switch to app {}'s page table.",app_index);
   let page_table_root = APP_PT_SV39[app_index].as_ptr() as usize -
   axconfig::PHYS_VIRT_OFFSET;

   KERNERL_PAGE_ROOT=satp::read().bits()<<12;
   satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
   riscv::asm::sfence_vma_all();
}

 fn load_and_run_apps(){
   let header_start=PLASH_START as *const u8;
   let apps_amount_byte=unsafe {
      core::slice::from_raw_parts(header_start, HEADER_LENGTH)  
   };
   let apps_amount=bytes_to_usize(apps_amount_byte);
   //  apps_start = PLASH_START  +  size of app_amount (2 bit)  + sizes of app_length(2 bit * apps_amount)
   let mut apps_start=PLASH_START+HEADER_LENGTH+ HEADER_LENGTH*apps_amount;
   let mut apps_run_start=RUN_START;

   unsafe{init_app_page_table(apps_amount);}
   for idx in 0..apps_amount{
      unsafe{switch_app_aspace(idx);}

      let apps_size_start: *const u8 = (PLASH_START+HEADER_LENGTH+idx) as *const u8;
      let apps_size_byte=unsafe {
          core::slice::from_raw_parts(apps_size_start, HEADER_LENGTH)
      };
      let app_size=bytes_to_usize(apps_size_byte);
      let load_code =unsafe {
          core::slice::from_raw_parts(apps_start as *const u8,app_size)
      };
      // apps_start+=app_size;
      // println!("app {} size:{:?} content:{:?}",idx,app_size,load_code);

      let run_code=unsafe {
          core::slice::from_raw_parts_mut(apps_run_start as *mut u8, app_size)
      };
      run_code.copy_from_slice(load_code);
      // println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

      println!("Execute app ...");

      println!("0x{:X}",satp::read().bits());
      // execute app
      unsafe { core::arch::asm!("
            la a0, {abi_table}
            mv      t2, {0}
            jalr    t2
            ",
            inout(reg) apps_run_start,
            abi_table=sym ABI_TABLE,
        )};
      // apps_run_start+=app_size;

      unsafe{switch_to_physical_page();}
      

   }

   


}
 #[inline]
 fn bytes_to_usize(bytes: &[u8]) -> usize {
   let mut res:usize=0;
   for &byte in bytes{
      res=res<<8;
      res+=byte as usize;
   }
   res
 }
 fn register_abi_table(){
   register_abi(SYS_HELLO, abi_hello as usize);
   register_abi(SYS_PUTCHAR, abi_putchar as usize);
   register_abi(SYS_TERMINATE, abi_terminate as usize);
   // unsafe { core::arch::asm!("
   // li      t0, {abi_num}
   // slli    t0, t0, 3
   // la      t1, {abi_table}
   // add     t1, t1, t0
   // ld      t1, (t1)
   // jalr    t1
   // li      t2, {run_start}
   // jalr    t2
   // j       .",
   // run_start = const RUN_START,
   // abi_table = sym ABI_TABLE,
   // //abi_num = const SYS_HELLO,
   // abi_num = const SYS_TERMINATE,
   // )}
 }