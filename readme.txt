========== Setup environment (Windows) =============
1. Open powershell administrator mode.
2. Set HOME environment variable.
  $Env:HOME=<Your project directory>
3. Delete "platform-tools" shortcut file.
  ex) Maybe exist on such below path.
  C:\Users\[user]\.local\share\solana\install\releases\1.18.9\solana-release\bin\sdk\sbf\dependencies
4. Run "cargo-build-bpf"
  Force exit the downloading process by Ctrl + C.
5. Extract platform-tools-windows-x86_64.tar.bz2.
  Extract it to ".cach/solana/v1.41/platform-tools/"
6. Rerun "cargo-build-bpf"
7. Enjoy!

========== Build & Deploy =============
1. Build
  cargo-build-bpf
2. Deploy
  - solana program deploy target/deploy/s1_moview_review.so
  or
  - solana program deploy --buffer ./recover.json target/deploy/s1_moview_review.so

========== Test =============
cd test
npm install
npm start
** Be remember to copy the program_id to proper position in "constants.ts" file.