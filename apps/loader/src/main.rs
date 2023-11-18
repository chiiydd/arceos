#![cfg_attr(feature = "axstd", no_std)]
 #![cfg_attr(feature = "axstd", no_main)]
 #![feature(asm_const)]
 #[cfg(feature = "axstd")]
 use axstd::println;
 const PLASH_START: usize = 0x22000000;
 const RUN_START: usize = 0xffff_ffc0_8010_0000;
 const HEADER_LENGTH:usize=1;
 #[cfg_attr(feature = "axstd", no_mangle)]

 fn main() {
    // let header_start=PLASH_START as *const u8;
    // let apps_start: *const u8 = (PLASH_START+HEADER_LENGTH) as *const u8;
    // // let apps_size = 32; // Dangerous!!! We need to get accurate size of apps.
    // let apps_size_bytes:&[u8]=unsafe {core::slice::from_raw_parts(header_start, HEADER_LENGTH)};


    // let apps_size=apps_size_bytes[0] as usize;
   
    println!("Load payload ...");

    load_and_run_apps();
    // let code = unsafe { core::slice::from_raw_parts(apps_start, apps_size) };
    // println!("app size:{:?} content:{:?}",apps_size,code);
    // println!("Load payload ok!");
 }

 fn load_and_run_apps(){
   let header_start=PLASH_START as *const u8;
   let apps_amount_byte=unsafe {
      core::slice::from_raw_parts(header_start, HEADER_LENGTH)  
   };
   let apps_amount=apps_amount_byte[0] as usize;
   //  apps_start = PLASH_START  +  size of app_amount (1 bit)  + sizes of app_length(1 bit * apps_amount)
   let mut apps_start=PLASH_START+HEADER_LENGTH+ HEADER_LENGTH*apps_amount;
   let mut apps_run_start=RUN_START;
   for idx in 0..apps_amount{
      let apps_size_start: *const u8 = (PLASH_START+HEADER_LENGTH+idx) as *const u8;
      let apps_size_byte=unsafe {
          core::slice::from_raw_parts(apps_size_start, HEADER_LENGTH)
      };
      let app_size=apps_size_byte[0] as usize;
      let load_code =unsafe {
          core::slice::from_raw_parts(apps_start as *const u8,app_size)
      };
      apps_start+=app_size;
      println!("app {} size:{:?} content:{:?}",idx,app_size,load_code);

      let run_code=unsafe {
          core::slice::from_raw_parts_mut(apps_run_start as *mut u8, app_size)
      };
      run_code.copy_from_slice(load_code);
      println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

      println!("Execute app ...");
      // execute app
      unsafe { core::arch::asm!("
            mv      t2, {0}
            jalr    t2
            ",
            inout(reg) apps_run_start
        )};
      apps_run_start+=app_size;
      

   }

   


}
 #[inline]
 fn bytes_to_usize(bytes: &[u8]) -> usize {
    usize::from_be_bytes(bytes.try_into().unwrap())
 }