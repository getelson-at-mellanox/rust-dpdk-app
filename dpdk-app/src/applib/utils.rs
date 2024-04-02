use std::ffi::CStr;
use std::os::raw::c_char;
use crate::Port;
use dpdklib::rte_ethdev::{rte_eth_dev_get_name_by_port, rte_eth_dev_info, rte_eth_dev_info_get,
                          RTE_ETH_NAME_MAX_LEN, RTE_ETH_RSS_IP,
                          rte_eth_rx_mq_mode_RTE_ETH_MQ_RX_RSS,
                          rte_eth_rx_mq_mode_RTE_ETH_MQ_RX_VMDQ_DCB_RSS};

// pub const PORT_ANY:DpdkPort = !0 as DpdkPort;
//
// pub unsafe fn is_valid_port_id(port_id:DpdkPort) -> bool {
//     if port_id == PORT_ANY { return true }
//     else {
//         for id in iter_rte_eth_dev().take(raw::rte_build_config::RTE_MAX_ETHPORTS as usize) {
//             if id == port_id { return true }
//         }
//     }
//     false
// }

pub unsafe fn init_port_config(port: &mut Port) {
    let ret = rte_eth_dev_info_get(port.port_id, &mut port.dev_info as *mut rte_eth_dev_info);
    if ret != 0 {
        panic!("port-{}: failed to get dev info {ret}", port.port_id);
    }

    port.dev_conf.rx_adv_conf.rss_conf.rss_key = std::ptr::null_mut();
    port.dev_conf.rx_adv_conf.rss_conf.rss_hf = if port.rxq_num > 1 {
        RTE_ETH_RSS_IP as u64 & port.dev_info.flow_type_rss_offloads
    } else {0};

    if port.dev_conf.rx_adv_conf.rss_conf.rss_hf != 0 {
        port.dev_conf.rxmode.mq_mode =
            rte_eth_rx_mq_mode_RTE_ETH_MQ_RX_VMDQ_DCB_RSS & rte_eth_rx_mq_mode_RTE_ETH_MQ_RX_RSS;
    }


}

pub unsafe fn show_ports_summary(ports: &Vec<Port>) {
    let mut name_buf:[c_char;RTE_ETH_NAME_MAX_LEN as usize]= [0 as c_char;RTE_ETH_NAME_MAX_LEN as usize];
    let title = format!("{:<4}    {:<32} {:<14}", "Port", "Name", "Driver");
    println!("{title}");
    ports.iter().for_each(|p| unsafe {
        let _rc = rte_eth_dev_get_name_by_port(p.port_id, name_buf.as_mut_ptr());
        let name = CStr::from_ptr(name_buf.as_ptr());
        let drv = CStr::from_ptr(p.dev_info.driver_name);
        let summary = format!("{:<4}    {:<32} {:<14}",
                              p.port_id, name.to_str().unwrap(), drv.to_str().unwrap());
        println!("{summary}");
    });

}