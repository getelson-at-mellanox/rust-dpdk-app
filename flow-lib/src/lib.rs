use std::ffi::c_void;

use dpdk_common::{FlowItems, FlowActions, init_struct,};
use dpdklib::rte_ethdev::{
    rte_flow_attr, rte_flow_action,
    rte_flow_item_type_RTE_FLOW_ITEM_TYPE_ETH, rte_flow_item_type_RTE_FLOW_ITEM_TYPE_IPV4,
    rte_flow_item_type_RTE_FLOW_ITEM_TYPE_UDP, rte_flow_item_type_RTE_FLOW_ITEM_TYPE_END,
    rte_flow_action_type_RTE_FLOW_ACTION_TYPE_PORT_ID,
    rte_flow_action_type_RTE_FLOW_ACTION_TYPE_END,
    rte_flow_action_port_id,
};

unsafe fn base_attr() -> rte_flow_attr {
    let mut attr = init_struct::<rte_flow_attr>();
    attr.set_transfer(1);
    attr
}

unsafe fn base_pattern() -> FlowItems {
    let mut pattern:FlowItems = init_struct::<FlowItems>();

    pattern[0].type_ = rte_flow_item_type_RTE_FLOW_ITEM_TYPE_ETH;
    pattern[1].type_ = rte_flow_item_type_RTE_FLOW_ITEM_TYPE_IPV4;
    pattern[2].type_ = rte_flow_item_type_RTE_FLOW_ITEM_TYPE_UDP;
    pattern[3].type_ = rte_flow_item_type_RTE_FLOW_ITEM_TYPE_END;
    pattern
}

unsafe fn base_actions() -> FlowActions {
    let mut actions:FlowActions = init_struct::<FlowActions>();
    let mut port_conf = init_struct::<rte_flow_action_port_id>();
    port_conf.id = 1;

    actions[0] = rte_flow_action {
        type_: rte_flow_action_type_RTE_FLOW_ACTION_TYPE_PORT_ID,
        conf: (&port_conf as *const rte_flow_action_port_id) as *const c_void
    };
    actions[1] = rte_flow_action {
        type_: rte_flow_action_type_RTE_FLOW_ACTION_TYPE_END,
        conf: 0 as *const c_void
    };
    actions
}

pub unsafe fn flow_params() -> (Vec<rte_flow_attr>, Vec<FlowItems>, Vec<FlowActions>) {
    let mut attributes:Vec<rte_flow_attr> = vec![];
    let mut patterns:Vec<FlowItems> = vec![];
    let mut actions:Vec<FlowActions> = vec![];

    attributes.push(base_attr());
    patterns.push(base_pattern());
    actions.push(base_actions());
    (attributes, patterns, actions)
}