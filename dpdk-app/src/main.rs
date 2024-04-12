mod applib;

use std::env;
use std::ffi::{c_void, CString};
use std::os::raw::{c_int, c_char};
use std::{thread, time};
use dpdklib::rte_ethdev::
{rte_eth_dev_info, rte_eth_conf, rte_eth_dev_configure, rte_eth_dev_start, rte_eth_tx_queue_setup,
 rte_eth_rx_queue_setup, rte_pktmbuf_pool_create, rte_eal_init, rte_exit, rte_eth_txconf, rte_mempool,
 rte_eth_rxconf, RTE_MBUF_DEFAULT_BUF_SIZE, rte_flow_attr, rte_flow_item, rte_flow_error,
 rte_flow_item_eth, RTE_ETHER_TYPE_IPV4,
 rte_flow_item_type_RTE_FLOW_ITEM_TYPE_END, rte_flow_item_type_RTE_FLOW_ITEM_TYPE_ETH,
 rte_flow_item_type_RTE_FLOW_ITEM_TYPE_IPV4, rte_flow_item_type_RTE_FLOW_ITEM_TYPE_UDP,
 rte_flow_action, rte_flow_action_type_RTE_FLOW_ACTION_TYPE_END,
 rte_be16_t, rte_flow_action_type_RTE_FLOW_ACTION_TYPE_PORT_ID,
 rte_flow_action_port_id, rte_flow_create, rte_flow};
use crate::applib::rte_api::{DpdkPort, iter_rte_eth_dev};
use crate::applib::utils::{init_port_config, show_ports_summary};


struct Port {
    port_id:DpdkPort,
    dev_info:rte_eth_dev_info,
    dev_conf:rte_eth_conf,
    rxq_num:u16,
    txq_num:u16,
}

impl Port {
    unsafe fn new(id:DpdkPort) -> Self {
        Port {
            port_id:id,
            dev_info: {
                let uninit: ::std::mem::MaybeUninit<rte_eth_dev_info> = ::std::mem::MaybeUninit::zeroed().assume_init();
                *uninit.as_ptr()
            },
            dev_conf:{
                let uninit: ::std::mem::MaybeUninit<rte_eth_conf> = ::std::mem::MaybeUninit::zeroed().assume_init();
                *uninit.as_ptr()
            },
            rxq_num:1,
            txq_num:1,
        }
    }
}

fn main() {
    let mut argv: Vec<*mut c_char> = env::args().map(|arg| CString::new(arg).unwrap().into_raw()).collect();

    let rc = unsafe { rte_eal_init(env::args().len() as c_int, argv.as_mut_ptr()) };
    if rc == -1 {
        let msg = CString::new("oops").unwrap();
        unsafe { rte_exit(255 as c_int,  msg.as_ptr()); }
    }
    let _eal_params_num = rc as usize;

    let mut ports:Vec<Port> = vec![];
    unsafe {
        for port_id in iter_rte_eth_dev()
            .take(dpdklib::rte_build_config::RTE_MAX_ETHPORTS as usize) {
            let mut port = Port::new(port_id);
            init_port_config(&mut port);
            println!("init port {port_id}");
            start_port(&mut port);
            ports.push(port);
        }
    }

    unsafe { show_ports_summary(&ports); }
    unsafe { flow_create(&ports[0])}
}

unsafe fn start_port(port:&mut Port) {
    let mut rc = rte_eth_dev_configure(port.port_id, port.rxq_num, port.txq_num,
                                       &port.dev_conf as *const rte_eth_conf);
    if rc != 0 { panic!("failed to configure port-{}: {rc}", port.port_id)}
    println!("port-{} configured", port.port_id);

    rc = rte_eth_tx_queue_setup(port.port_id, 0, 64, 0, 0 as *const rte_eth_txconf);
    if rc != 0 { panic!("port-{}: failed to configure TX queue 0 {rc}", port.port_id)}
    println!("port-{} configured TX queue 0", port.port_id);

    let mbuf_pool_name = CString::new(format!("mbuf pool port-{}", port.port_id)).unwrap();
    let mbuf_pool = rte_pktmbuf_pool_create(
        mbuf_pool_name.as_ptr(), 1024, 0, 0,
        RTE_MBUF_DEFAULT_BUF_SIZE as u16, 0);
    if mbuf_pool == 0 as *mut rte_mempool {
        panic!("port-{}: failed to allocate mempool {rc}", port.port_id)
    }
    println!("port-{} mempool ready", port.port_id);

    let mut rxq_conf:rte_eth_rxconf = port.dev_info.default_rxconf.clone();
    rxq_conf.offloads = 0;
    rc = rte_eth_rx_queue_setup(port.port_id, 0, 64, 0,
                                &mut rxq_conf as *mut rte_eth_rxconf, mbuf_pool);
    if rc != 0 { panic!("port-{}: failed to configure RX queue 0 {rc}", port.port_id)}
    println!("port-{} configured RX queue 0", port.port_id);
    rc = rte_eth_dev_start(port.port_id);
    if rc != 0 { panic!("failed to start port-{}: {rc}", port.port_id)}
    println!("port-{} started", port.port_id);
}

unsafe fn init_struct_ptr<T>() -> ::std::mem::MaybeUninit<T> {
    let uninit: ::std::mem::MaybeUninit<T> = ::std::mem::MaybeUninit::zeroed().assume_init();
    uninit

}

unsafe fn init_struct<T: Copy>() -> T {
    *init_struct_ptr::<T>().as_ptr()
}

unsafe fn flow_create(port:&Port) {
    let err = init_struct::<rte_flow_error>();
    let mut attr = init_struct::<rte_flow_attr>();
    attr.set_transfer(1);
    let mut port_conf = init_struct::<rte_flow_action_port_id>();
    port_conf.id = 1;

    let mut eth_spec = init_struct::<rte_flow_item_eth>();
    let mut eth_mask = init_struct::<rte_flow_item_eth>();
    eth_spec.__bindgen_anon_1.hdr.ether_type = (RTE_ETHER_TYPE_IPV4 as u16).to_be();
    eth_mask.__bindgen_anon_1.hdr.ether_type = 0xffff as rte_be16_t;

    let pattern= [
        rte_flow_item {
            type_:rte_flow_item_type_RTE_FLOW_ITEM_TYPE_ETH,
            spec: (&eth_spec as *const rte_flow_item_eth) as *const c_void,
            mask: (&eth_mask as *const rte_flow_item_eth) as *const c_void,
            last: 0 as *const c_void,
        },
        rte_flow_item {
            type_:rte_flow_item_type_RTE_FLOW_ITEM_TYPE_IPV4,
            spec: 0 as *const c_void,
            last:0 as *const c_void,
            mask:0 as *const c_void,
        },
        rte_flow_item {
            type_:rte_flow_item_type_RTE_FLOW_ITEM_TYPE_UDP,
            spec: 0 as *const c_void,
            last:0 as *const c_void,
            mask:0 as *const c_void,
        },
        rte_flow_item {
            type_:rte_flow_item_type_RTE_FLOW_ITEM_TYPE_END,
            spec: 0 as *const c_void,
            last:0 as *const c_void,
            mask:0 as *const c_void,
        },
    ];

    let actions = [
        rte_flow_action {
            type_: rte_flow_action_type_RTE_FLOW_ACTION_TYPE_PORT_ID,
            conf: (&port_conf as *const rte_flow_action_port_id) as *const c_void
        },
        rte_flow_action {
            type_: rte_flow_action_type_RTE_FLOW_ACTION_TYPE_END,
            conf: 0 as *const c_void
        },
    ];

    let flow = rte_flow_create(port.port_id, &attr as *const rte_flow_attr,
                               &pattern as *const rte_flow_item,
                               &actions as *const rte_flow_action,
                               (&err as *const rte_flow_error) as *mut rte_flow_error);

    if flow == 0 as *mut rte_flow {
        println!("failed to create a flow")
    }

    println!("created a flow");
    let delay = time::Duration::from_secs(1);
    loop {
        thread::sleep(delay);
    }
}

