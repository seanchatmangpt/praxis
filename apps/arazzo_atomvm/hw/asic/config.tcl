# OpenLANE Configuration for Arazzo Silicon Tape-Out (TSMC / Generic Target)

set ::env(DESIGN_NAME) "arazzo_workflow_fsm"

# Source files
set ::env(VERILOG_FILES) [glob $::env(DESIGN_DIR)/../arazzo_workflow_fsm.v]

# Clock Configuration
# Targeting 500MHz (2.0ns period) for pure combinatorial bitmask speed
set ::env(CLOCK_PORT) "clk"
set ::env(CLOCK_PERIOD) 2.0

# Core Floorplan and Placement Density
# 64-bit bitmask logic is small, use high placement density
set ::env(FP_SIZING) absolute
set ::env(DIE_AREA) "0 0 100 100"
set ::env(CORE_AREA) "5 5 95 95"
set ::env(PL_TARGET_DENSITY) 0.80
set ::env(FP_PDN_CORE_RING) 1

# ASIC Synthesis Strategy
set ::env(SYNTH_STRATEGY) "AREA 0"
set ::env(SYNTH_MAX_FANOUT) 20

# GDSII Generation (Final physical layout mask)
set ::env(MAGIC_GENERATE_GDS) 1
set ::env(ROUTING_CORES) 8
