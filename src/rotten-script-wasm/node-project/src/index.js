"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var _a;
Object.defineProperty(exports, "__esModule", { value: true });
const rotten_script_wasm_1 = require("../../pkg/rotten_script_wasm");
const fs_1 = __importDefault(require("fs"));
const helpText = "Usage: npm start -- [filename|dirName] [-d]";
const args = process.argv;
const getDirectoryRecursive = (path) => {
    const paths = [];
    const dirents = fs_1.default.readdirSync(path, { withFileTypes: true });
    dirents.forEach((x) => {
        if (x.isDirectory()) {
            paths.push(...getDirectoryRecursive(`${path}/${x.name}`));
        }
        if (x.isFile() && x.name.endsWith(".rots")) {
            paths.push(`${path}/${x.name}`);
        }
    });
    return paths;
};
if (args.length < 3) {
    console.log(helpText);
}
else {
    if (args.length > 3 && args[3] === "-d") {
        const files = getDirectoryRecursive(args[2]);
        for (const item of files) {
            const text = fs_1.default.readFileSync(item, "utf-8");
            rotten_script_wasm_1.add_file(item, text);
        }
        rotten_script_wasm_1.execute_processing();
        if (fs_1.default.existsSync(`${args[2]}/dist`)) {
            fs_1.default.rmSync(`${args[2]}/dist`, { recursive: true });
        }
        fs_1.default.mkdirSync(`${args[2]}/dist`);
        for (const item of files) {
            const last_slash = item.lastIndexOf("/");
            const dir_name = item.slice(0, last_slash);
            const target = `${args[2]}/dist/${dir_name}`;
            if (!fs_1.default.existsSync(target)) {
                fs_1.default.mkdirSync(target);
            }
            const deletedRots = (_a = item.match(/^(.*)\.rots$/)) === null || _a === void 0 ? void 0 : _a[1];
            if (deletedRots) {
                fs_1.default.writeFileSync(`${args[2]}/dist/${deletedRots}.js`, rotten_script_wasm_1.eject_sourcecode(item));
            }
        }
    }
    else {
        const text = fs_1.default.readFileSync(args[2], "utf-8");
        rotten_script_wasm_1.process(text);
    }
}
//# sourceMappingURL=index.js.map