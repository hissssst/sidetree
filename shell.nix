with (import <nixpkgs> {});
pkgs.mkShell rec {
  buildInputs = with pkgs; [ cargo rustc ];

  NIX_ENFORCE_PURITY = 0;
  RUSTC_BOOTSTRAP = 1;
}
