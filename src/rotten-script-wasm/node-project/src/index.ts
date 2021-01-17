import {
  process as wasmProcess,
  add_file,
  execute_processing,
  eject_sourcecode,
} from "../../pkg/rotten_script_wasm";
import fs from "fs";

const helpText = "Usage: npm start -- [filename|dirName] [-d]";

const args = process.argv;

const getDirectoryRecursive = (path: string): string[] => {
  const paths: string[] = [];
  const dirents = fs.readdirSync(path, { withFileTypes: true });
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
} else {
  if (args.length > 3 && args[3] === "-d") {
    const files = getDirectoryRecursive(args[2]);
    for (const item of files) {
      const text = fs.readFileSync(item, "utf-8");
      add_file(item, text);
    }
    execute_processing();
    fs.rmSync(`${args[2]}/dist`, { recursive: true });
    fs.mkdirSync(`${args[2]}/dist`);
    for (const item of files) {
      const last_slash = item.lastIndexOf("/");
      const dir_name = item.slice(0, last_slash);
      const target = `${args[2]}/dist/${dir_name}`;
      if (!fs.existsSync(target)) {
        fs.mkdirSync(target);
      }
      const deletedRots = item.match(/^(.*)\.rots$/)?.[1];
      if (deletedRots) {
        fs.writeFileSync(
          `${args[2]}/dist/${deletedRots}.js`,
          eject_sourcecode(item)
        );
      }
    }
  } else {
    const text = fs.readFileSync(args[2], "utf-8");
    wasmProcess(text);
  }
}
