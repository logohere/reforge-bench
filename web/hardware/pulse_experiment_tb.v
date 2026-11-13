`timescale 1ns/1ns
module pulse_experiment_tb;
    reg clk = 0, reset_button = 1, stimulus = 0;
    wire heartbeat, led_green, fault;
    integer scenario, sample_file, cycle;
    pulse_loop #(.WATCHDOG_CYCLES(100000)) dut(
        .clk(clk), .reset_button(reset_button), .stimulus(stimulus),
        .heartbeat(heartbeat), .led_green(led_green), .fault(fault)
    );
    always #500 clk = !clk;
    always @(stimulus or heartbeat or fault or reset_button) begin
        #1;
        $fdisplay(sample_file, "%0d,%0b,%0b,%0b,%0b", $time, stimulus, heartbeat, fault, reset_button);
    end
    initial begin
        if (!$value$plusargs("scenario=%d", scenario)) $fatal(1, "Missing scenario");
        sample_file = $fopen("trace.csv", "w");
        $fdisplay(sample_file, "time_ns,stimulus,heartbeat,fault,reset");
        #10000 reset_button = 0;
    end
    initial begin
        #1150000000;
        if (scenario == 3) reset_button = 1;
        #50000000;
        if (scenario == 3) reset_button = 0;
    end
    initial begin
        for (cycle = 0; cycle < 2600; cycle = cycle + 1) begin
            #500000;
            if (cycle >= 1100 && cycle < 1350 && scenario == 1) stimulus = 0;
            else stimulus = 1;
            #500000;
            if (!(cycle >= 1100 && cycle < 1350 && scenario == 2)) stimulus = 0;
        end
        #10000;
        $fclose(sample_file);
        $finish;
    end
endmodule
