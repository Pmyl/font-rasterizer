import opentype from "opentype.js/dist/opentype.js";
import fs from "fs";

const buffer = fs.readFileSync("./Verdana.ttf");

const font = opentype.parse(buffer);
console.log(font.glyphs.glyphs[4].path);
