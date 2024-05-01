use dpdklib::rte_ethdev::{
    rte_flow_item, rte_flow_action,
};

pub const DPDK_MAX_FLOW_ITEMS:usize = 16;
pub const DPDK_MAX_FLOW_ACTIONS:usize = 16;

pub type FlowItems = [rte_flow_item;DPDK_MAX_FLOW_ITEMS];
pub type FlowActions = [rte_flow_action;DPDK_MAX_FLOW_ACTIONS];

pub unsafe fn init_struct_ptr<T>() -> ::std::mem::MaybeUninit<T> {
    let uninit: ::std::mem::MaybeUninit<T> = ::std::mem::MaybeUninit::zeroed().assume_init();
    uninit

}

pub unsafe fn init_struct<T: Copy>() -> T {
    *init_struct_ptr::<T>().as_ptr()
}