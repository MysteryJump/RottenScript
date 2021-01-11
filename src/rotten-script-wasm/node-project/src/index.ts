import { process as wasmProcess } from "../../pkg/rotten_script_wasm";
import fs from "fs";

const helpText = "Usage: npm start -- [filename]";

const args = process.argv;

const getDirectoryRecursive = (path: string): string[] => {
  const paths: string[] = [];
  const dirents = fs.readdirSync(path, { withFileTypes: true });
  dirents.forEach((x) => {
    if (x.isDirectory()) {
      paths.push(...getDirectoryRecursive(`${path}/${x.name}`));
    }
    if (x.isFile()) {
      paths.push(`${path}/${x.name}`);
    }
  });
  return paths;
};

if (args.length < 3) {
  console.log(helpText);
  const files = getDirectoryRecursive("sample");
  files.forEach((x) => console.log(x));
} else {
  const text = fs.readFileSync(args[2], "utf-8");
  wasmProcess(text);
}
