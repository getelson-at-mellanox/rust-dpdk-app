use dpdklib as raw;

pub type DpdkPort = u16;
pub unsafe fn iter_rte_eth_dev_owned_by(owner_id:u64) -> impl Iterator<Item=DpdkPort> {
    let mut port_id:DpdkPort = 0 as DpdkPort;
    std::iter::from_fn(move || {
        let cur = port_id;
        port_id = raw::rte_ethdev::rte_eth_find_next_owned_by(cur, owner_id) as DpdkPort;
        if port_id == raw::rte_build_config::RTE_MAX_ETHPORTS as DpdkPort {
            return None
        }
        if cur == port_id { port_id += 1 }
        Some(cur)
    })
}

pub unsafe fn iter_rte_eth_dev() -> impl Iterator<Item=DpdkPort> {
    iter_rte_eth_dev_owned_by(raw::rte_ethdev::RTE_ETH_DEV_NO_OWNER as u64)
}