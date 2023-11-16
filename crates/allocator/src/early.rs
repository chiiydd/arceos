

use core::ptr::NonNull;


use crate::{BaseAllocator, align_down, align_up, PageAllocator, AllocError, ByteAllocator,  AllocResult};


pub struct  EarlyAllocator<const PAGE_SIZE:usize>{

    base: usize,
    end:usize,
    byte_pos:usize,
    page_pos:usize,
    total_pages:usize,
    used_pages:usize,
    used_bytes:usize,
}


impl <const PAGE_SIZE:usize> EarlyAllocator<PAGE_SIZE>{
    pub const fn new() ->Self{
        Self { 
            base:0, 
            end: 0,
            byte_pos:0,
            page_pos:0,
            total_pages:0,
            used_pages:0,
            used_bytes:0,
         }
    }
}
impl<const PAGE_SIZE:usize> BaseAllocator for EarlyAllocator<PAGE_SIZE>{
    fn init(&mut self, start: usize, size: usize) {
    // self.base=align_down(start, PAGE_SIZE);
    self.base=start;
    self.end= align_up(start+size, PAGE_SIZE);
    self.byte_pos=self.base;
    self.page_pos=self.end;
    self.total_pages= (self.end-self.base)/PAGE_SIZE;
    }
    fn add_memory(&mut self, start: usize, size: usize) -> crate::AllocResult {
        Err(AllocError::MemoryOverlap)
    }
}
impl <const PAGE_SIZE:usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

     fn total_pages(&self) -> usize {
        self.total_pages
    }
    fn used_pages(&self) -> usize {
        self.used_pages
    }
    fn available_pages(&self) -> usize {
        self.total_pages-self.used_pages
    }
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> crate::AllocResult<usize> {
        if align_pow2 % PAGE_SIZE != 0 {
            return Err(AllocError::InvalidParam);
        }
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }

        if num_pages==0{
            return Err(AllocError::InvalidParam);
        }
        
        let size = (num_pages * PAGE_SIZE + align_pow2 - 1) & (!align_pow2 + 1);
        
        let new_page_pos=(self.page_pos-size)&(!align_pow2+1);
        if new_page_pos<self.byte_pos{
            return Err(AllocError::NoMemory);
        }
        self.used_pages+=(self.page_pos-new_page_pos)/PAGE_SIZE;
        
        self.page_pos=new_page_pos;
        
        Ok(self.page_pos)

    }
    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        if _pos>=self.page_pos&&_pos<self.end{
            self.used_pages-=_num_pages;
            if  self.used_pages==0{
                self.page_pos=self.end;
            }
        }
    }
}

impl <const PAGE_SIZE:usize> ByteAllocator for EarlyAllocator<PAGE_SIZE>{
    fn alloc(&mut self, layout: core::alloc::Layout) -> crate::AllocResult<core::ptr::NonNull<u8>> {
        if  !layout.align().is_power_of_two(){
            return Err(AllocError::InvalidParam);
        }
        let start=(self.byte_pos+layout.align()-1)&(!layout.align()+1);
        let  alloc_pos=(start+layout.size()+layout.align()-1)&(!layout.align()+1);
        
        if alloc_pos>self.page_pos{
            return Err(AllocError::NoMemory);
        }
    
        self.used_bytes+=alloc_pos-start;
        self.byte_pos=alloc_pos;
        


        let result=NonNull::new(start as *mut u8 ).unwrap();
        return AllocResult::Ok(result);


    }
    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        // self.byte_pos-= layout.size();
        self.used_bytes-=layout.size();
    }
    fn available_bytes(&self) -> usize {
        self.page_pos-self.byte_pos
    }
    fn total_bytes(&self) -> usize {
        self.page_pos-self.base
    }
    fn used_bytes(&self) -> usize {
        self.used_bytes
    }
}