`timescale 1ns/1ps
module pulse_loop_tb;
    reg clk = 0, reset_button = 1, stimulus = 1;
    wire heartbeat, led_green, fault;
    pulse_loop #(.WATCHDOG_CYCLES(100)) dut(clk, reset_button, stimulus, heartbeat, led_green, fault);
    always #4 clk = !clk;
    task pulse;
        begin
            @(negedge clk); stimulus = 1;
            repeat(5) @(negedge clk);
            stimulus = 0;
            repeat(5) @(negedge clk);
        end
    endtask
    integer i;
    initial begin
        repeat(5) @(negedge clk);
        reset_button = 0;
        repeat(5) @(negedge clk);
        if (heartbeat !== 0 || fault !== 1) $fatal(1, "reset state");
        if (dut.edge_count !== 0) $fatal(1, "startup high fabricated an edge");
        stimulus = 0;
        repeat(5) @(negedge clk);
        for (i = 0; i < 999; i = i + 1) begin
            pulse();
            if (heartbeat !== 0 || fault !== 0) $fatal(1, "early toggle or unexpected fault");
        end
        pulse();
        if (heartbeat !== 1 || led_green !== 1) $fatal(1, "1000-edge toggle");
        for (i = 0; i < 1000; i = i + 1) pulse();
        if (heartbeat !== 0) $fatal(1, "second 1000-edge toggle");
        stimulus = 1;
        repeat(20) @(negedge clk);
        if (dut.edge_count !== 1) $fatal(1, "held-high input counted more than once");
        repeat(100) @(negedge clk);
        if (fault !== 1 || heartbeat !== 0 || dut.edge_count !== 0) $fatal(1, "watchdog failure");
        stimulus = 0;
        repeat(10) @(negedge clk);
        pulse();
        if (fault !== 0 || dut.edge_count !== 1) $fatal(1, "watchdog recovery");
        reset_button = 1;
        repeat(10) @(negedge clk);
        if (fault !== 1 || heartbeat !== 0 || dut.edge_count !== 0) $fatal(1, "button reset");
        stimulus = 1;
        repeat(5) @(negedge clk);
        reset_button = 0;
        repeat(10) @(negedge clk);
        if (fault !== 1 || dut.edge_count !== 0) $fatal(1, "reset release high fabricated an edge");
        stimulus = 0;
        repeat(5) @(negedge clk);
        pulse();
        if (fault !== 0 || dut.edge_count !== 1) $fatal(1, "post-reset low/rise recovery");
        $display("PASS: 1000-edge division, no early toggle, held-high input, watchdog, recovery, reset");
        $finish;
    end
    initial begin
        #1000000;
        $fatal(1, "test timeout");
    end
endmodule
