import wasm from "../../pkg/rotten_script_wasm.js"
import fs from "fs";

const helpText = "Usage: npm start -- [filename]";

const args = process.argv;

if (args.length < 3) {
    console.log(helpText);
} else {
    const text = fs.readFileSync(args[2], "utf-8");
    wasm.process(text);
}

