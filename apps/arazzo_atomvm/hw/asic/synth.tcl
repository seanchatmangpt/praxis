# Yosys logic synthesis mapping for Arazzo Silicon Tape-Out
# Translates the Verilog FSM into standard cell logic targets

# Read design
read_verilog ../arazzo_workflow_fsm.v

# Elaborate design hierarchy
hierarchy -check -top arazzo_workflow_fsm

# High-level synthesis
proc
opt
fsm
opt
memory
opt

# Map to internal gate library (techmap)
techmap
opt

# DFF mapping
dfflibmap -liberty stdcells.lib

# Generic standard cell mapping
abc -liberty stdcells.lib

# Final optimizations and cleanup
opt_clean

# Output mapped structural netlist
write_verilog arazzo_workflow_fsm_mapped.v
write_json arazzo_workflow_fsm_mapped.json

stat
