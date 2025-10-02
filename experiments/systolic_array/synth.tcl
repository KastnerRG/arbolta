yosys -import

set top_module axis_sa

set rtl_path axis-systolic-array/rtl/sa
set rtl_files [glob -directory $rtl_path -- "*.sv" "**/*.sv"]

foreach verilog_source $rtl_files {
  read_verilog -defer -sv $verilog_source
}

# Overwrite module parameters
foreach param [lrange $argv 0 end] {
  set param [split $param "="]
  set param_name [lindex $param 0]
  set param_val [lindex $param 1]
  chparam -set $param_name $param_val $top_module
}

hierarchy -check -top $top_module

procs;;

proc export_synth {output_path} {
  file mkdir $output_path
  write_json $output_path/synth.json
  # write_verilog $output_path/synth.v
  # tee -o $output_path/torder.txt torder
  # show -viewer none -format dot -prefix $output_path/schematic
}

flatten

puts [export_synth {output/0_proc}]

# Synthesize design
synth -top $top_module
clean -purge
autoname

setundef -zero
puts [export_synth {output/1_synth}]

# read_verilog -lib cells/cells.v
# dfflibmap -liberty cells/cells.lib
# abc -liberty cells/cells.lib
# opt_clean

# puts [export_synth {output/2_cell_synth}]
