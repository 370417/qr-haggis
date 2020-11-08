import init, { run_app } from "./pkg/qr_haggis.js";
async function main() {
    await init("/pkg/qr_haggis_bg.wasm");
    run_app();
}
main();
