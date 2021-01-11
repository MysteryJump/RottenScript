"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
var rotten_script_wasm_1 = require("../../pkg/rotten_script_wasm");
var fs_1 = __importDefault(require("fs"));
var helpText = "Usage: npm start -- [filename]";
var args = process.argv;
if (args.length < 3) {
    console.log(helpText);
}
else {
    var text = fs_1.default.readFileSync(args[2], "utf-8");
    rotten_script_wasm_1.process(text);
}
