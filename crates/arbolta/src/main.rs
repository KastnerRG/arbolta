use arbolta::{
  bit::{Bit, BitVec},
  hardware_design::HardwareDesign,
  netlist_wrapper::NetlistWrapper,
  yosys::{Netlist, parse_torder},
};

static RAW_NETLIST: &[u8] = include_bytes!(
  "/Users/alexanderredding/Work/research/arbolta/arbolta/crates/arbolta/tests/deps/design_netlists/int2_mac_netlist.json"
);

static RAW_TORDER: &[u8] = include_bytes!(
  "/Users/alexanderredding/Work/research/arbolta/arbolta/crates/arbolta/tests/deps/design_netlists/int2_mac_torder.txt"
);

fn main() -> anyhow::Result<()> {
  let netlist = Netlist::from_slice(RAW_NETLIST)?;
  let torder = parse_torder(RAW_TORDER)?;
  let netlist_wrapper = NetlistWrapper::new(None, netlist, torder, None)?;
  let mut design = HardwareDesign::new(netlist_wrapper, None)?;

  // // Set clock + reset
  let clock_net = design.get_net("clk_i").unwrap()[0];
  let reset_net = design.get_net("rst_ni").unwrap()[0];
  design.set_clock(clock_net, Bit::ONE)?; // posedge
  design.set_reset(reset_net, Bit::ZERO)?; // negedge

  // Start simulation
  design.eval_reset_clocked(None)?;
  design.set_port("en_i", [Bit::ONE])?;
  design.set_port("op0_i", BitVec::from_int(1, None))?;
  design.set_port("op1_i", BitVec::from_int(1, None))?;
  design.eval_clocked(Some(2))?;
  let result: i32 = design.get_port("acc_o").unwrap().to_int();
  assert_eq!(result, 2);

  Ok(())
}
