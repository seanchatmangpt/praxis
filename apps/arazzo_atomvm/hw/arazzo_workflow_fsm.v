`timescale 1ns / 1ps

/*
 * Arazzo Workflow FSM - Silicon Synthesis Target
 * 
 * This module transpiles the Erlang zero-allocation tail-call loop from
 * arazzo_atomvm_workflow and air_core into a synthesizable Verilog module.
 * 
 * The 64-bit state_mask replaces the ActiveSteps list, operating 
 * purely on bitwise logic (AND, NOT, OR) in hardware, executing state 
 * transitions in a single clock cycle.
 */
module arazzo_workflow_fsm (
    input wire clk,
    input wire rst_n,
    
    // Event Interface
    input wire event_valid,
    input wire [1:0] event_type,          // 2'b00: idle, 2'b01: step_completed, 2'b10: step_failed, 2'b11: stop
    input wire [63:0] event_step_bitmask, // One-hot encoded bitmask for the current step (1 << step_idx)
    input wire [63:0] event_next_bitmask, // Bitmask for the next steps to activate
    
    // Output State
    output reg [63:0] state_mask,
    output reg workflow_error,
    output reg workflow_done
);

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state_mask <= 64'b0;
            workflow_error <= 1'b0;
            workflow_done <= 1'b0;
        end else begin
            if (event_valid) begin
                case (event_type)
                    2'b01: begin // step_completed
                        // Equivalent to Erlang: 
                        // Mask1 = Mask band (bnot StepBit),
                        // Mask2 = Mask1 bor NextMask
                        state_mask <= (state_mask & ~event_step_bitmask) | event_next_bitmask;
                    end
                    2'b10: begin // step_failed
                        // Equivalent to Erlang: 
                        // Mask1 = Mask band (bnot StepBit)
                        state_mask <= (state_mask & ~event_step_bitmask);
                        workflow_error <= 1'b1;
                    end
                    2'b11: begin // stop
                        workflow_done <= 1'b1;
                    end
                    default: begin
                        // idle / unknown event - retain state
                    end
                endcase
            end
        end
    end

endmodule
