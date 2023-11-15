
#![no_std]

extern  crate alloc;
extern  crate axlog;
use core::fmt::Debug;

use core::str;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use axlog::debug;
use hermit_dtb::Dtb;


pub enum DtbError {
    FAIL
}
impl Debug for DtbError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f,"Error:Fail")
    }
}
pub type PaseDtbResult<T = DtbInfo> = core::result::Result<T, DtbError>;

pub struct DtbInfo {
    pub memory_addr: usize,
    pub memory_size: usize,
    pub mmio_regions: Vec<(usize, usize)>,
}

fn bytes_to_addr_size(bytes: &[u8]) -> (usize,usize) {
    
    let (addr_bytes,size_bytes)=bytes.split_at(bytes.len()/2);
    let mut addr=0usize;
    let mut size=0usize;
    for &byte in addr_bytes{
        addr=256*addr+byte as usize;
    }
    for &byte in size_bytes{
        size=256*size+byte as usize;
    }
    (addr,size)
}
fn recusion(dtb:&Dtb,path:&str){

    debug!("------------------");
    debug!("Node:{}",path);
    for property in dtb.enum_properties(path){
        debug!("property:{}",property);
        let data_option = dtb.get_property(path, property);
        debug!("{:?}", data_option);
        if let Some(data) = data_option {

            if let Ok(string) = str::from_utf8(data) {
                debug!("As string: \"{}\"", string);
            }
            // if  property.starts_with("reg"){
            //     debug!("DDSDDSDSDSDS");
            //     let (addr,size)=data.split_at(data.len()/2);
            //     debug!("{}:{}",bytes_to_hex_string(addr),bytes_to_hex_string(size));
            // }
        }
    }
    debug!("--------------------\n");
    for node in dtb.enum_subnodes(path){
        
        recusion(dtb, format!("{}{}/",path,node).as_str())
    }
}
// 参考函数原型
pub fn parse_dtb(dtb_pa: usize) -> PaseDtbResult {

    let mut dtb_info=DtbInfo{
        memory_addr:0,
        memory_size:0,
        mmio_regions:Vec::new(),
    };
    unsafe{
        let dtb=Dtb::from_raw(dtb_pa as *const u8 ).expect("parse dtb fails");
        // let dtb_1=Dtb::from_raw(dtb_pa as *const u8 ).expect("parse dtb fails");
        // recusion(&dtb_1, "/");
        // parse 
        for sub_node in dtb.enum_subnodes("/"){
            // debug!("Sub node:{}: ",sub_node);
            let node_name=String::from(sub_node);
            if node_name.starts_with("memory"){
                let memory_node=format!("/{}/",node_name);
                let data_option=dtb.get_property(memory_node.as_str(), "reg");
                if let Some(data)=data_option{
                    
                    let (addr,size)= bytes_to_addr_size(data);
                    // debug!("ADDR:{} SIZE: {}",addr,size);
                    dtb_info.memory_addr=addr;
                    dtb_info.memory_size=size;
                   
                }
            }
            if node_name.starts_with("soc"){
                let sub_path=format!("/{}/",sub_node);
                for new_sub in dtb.enum_subnodes(&sub_path){
                    let new_sub_name=String::from(new_sub);
                    if new_sub_name.starts_with("virtio_mmio@"){
                        let (addr,size)=bytes_to_addr_size(dtb.get_property(format!("{}{}/",sub_path,new_sub).as_str(), "reg").unwrap());
                        
                        dtb_info.mmio_regions.push((addr,size));
                    }
                }

            }
            // let sub_path=format!("/{}/",sub_node);
            // for node in dtb.enum_subnodes(sub_path.as_str()){
            //     debug!("{}",node);

            //     for property in dtb.enum_subnodes(&format!("{}{}/",sub_path,node)){
            //         debug!("Property:{}",property);
            //     } 
                // let path=format!("/{}/",n);
                // let data_option = dtb.get_property(path.as_str(), node);
                // debug!("{:?}", data_option);
        
                // if let Some(data) = data_option {
                //     if let Ok(string) = str::from_utf8(data) {
                //         debug!("As string: \"{}\"", string);
                //     }
            //     }
            // }    
        }
    }
    return PaseDtbResult::Ok(dtb_info);
}
