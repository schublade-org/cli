const fs = require("node:fs");
const path = require("node:path");

const source = path.join(__dirname, "../ui/workshop.js");
const code = fs.readFileSync(source, "utf8");
const parts = 3;
const size = Math.ceil(code.length / parts);

for (let index = 0; index < parts; index += 1) {
  const chunk = code.slice(index * size, (index + 1) * size);
  fs.writeFileSync(path.join(__dirname, `../ui/workshop.${index}.js`), chunk);
}

console.log(`split ${code.length} bytes into ${parts} parts`);
