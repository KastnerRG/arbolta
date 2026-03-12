`include "/opt/oss-cad-suite/share/yosys/simlib.v"

// module not_wrapper (
//     A,
//     Y
// );

//   parameter A_SIGNED = 0;
//   parameter A_WIDTH = 0;
//   parameter Y_WIDTH = 0;

//   input [A_WIDTH-1:0] A;
//   output [Y_WIDTH-1:0] Y;

//   \$not #(
//       .A_SIGNED(A_SIGNED),
//       .A_WIDTH (A_WIDTH),
//       .Y_WIDTH (Y_WIDTH)
//   ) _TECHMAP_REPLACE_ (
//       .A(A),
//       .Y(Y)
//   );

// endmodule

module \$add_wrapper (
    A,
    B,
    Y
);

  parameter A_SIGNED = 0;
  parameter B_SIGNED = 0;
  parameter A_WIDTH = 0;
  parameter B_WIDTH = 0;
  parameter Y_WIDTH = 0;

  input [A_WIDTH-1:0] A;
  input [B_WIDTH-1:0] B;
  output [Y_WIDTH-1:0] Y;

  \$add #(
      .A_SIGNED(A_SIGNED),
      .B_SIGNED(B_SIGNED),
      .A_WIDTH (A_WIDTH),
      .B_WIDTH (B_WIDTH),
      .Y_WIDTH (Y_WIDTH)
  ) _TECHMAP_REPLACE_ (
      .A(A),
      .B(B),
      .Y(Y)
  );

endmodule
