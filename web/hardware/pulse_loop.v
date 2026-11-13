module pulse_loop #(
    parameter integer EDGES_PER_TOGGLE = 1000,
    parameter integer WATCHDOG_CYCLES = 12500000
)(
    input wire clk,
    input wire reset_button,
    input wire stimulus,
    output reg heartbeat = 0,
    output wire led_green,
    output reg fault = 1
);
    (* ASYNC_REG = "TRUE" *) reg reset_meta = 1, reset_sync = 1;
    (* ASYNC_REG = "TRUE" *) reg stim_meta = 0, stim_sync = 0;
    reg previous = 0;
    // After reset, observe a synchronized low before accepting a rising edge.
    // Keep the synchronizer running through reset: forcing it low invents an
    // edge when reset is released with the external input already high.
    reg input_armed = 0;
    localparam integer EDGE_WIDTH = EDGES_PER_TOGGLE > 1 ? $clog2(EDGES_PER_TOGGLE) : 1;
    localparam integer IDLE_WIDTH = WATCHDOG_CYCLES > 1 ? $clog2(WATCHDOG_CYCLES) : 1;
    reg [EDGE_WIDTH-1:0] edge_count = 0;
    reg [IDLE_WIDTH-1:0] idle_cycles = 0;
    assign led_green = heartbeat;

    always @(posedge clk) begin
        reset_meta <= reset_button;
        reset_sync <= reset_meta;
        stim_meta <= stimulus;
        stim_sync <= stim_meta;
        previous <= stim_sync;
        if (reset_sync) begin
            input_armed <= 0;
            edge_count <= 0;
            idle_cycles <= 0;
            heartbeat <= 0;
            fault <= 1;
        end else begin
            if (!stim_sync) input_armed <= 1;
            if (input_armed && stim_sync && !previous) begin
                idle_cycles <= 0;
                fault <= 0;
                if (edge_count == EDGES_PER_TOGGLE - 1) begin
                    edge_count <= 0;
                    heartbeat <= !heartbeat;
                end else edge_count <= edge_count + 1;
            end else if (idle_cycles >= WATCHDOG_CYCLES - 1) begin
                idle_cycles <= WATCHDOG_CYCLES - 1;
                edge_count <= 0;
                heartbeat <= 0;
                fault <= 1;
            end else idle_cycles <= idle_cycles + 1;
        end
    end
endmodule
