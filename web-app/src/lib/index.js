// place files you want to import through the `$lib` alias in this folder.
import { inflate } from "pako";
import { SaveData } from "redstone-wasm";

function base64ToBytes(base64) {
  const binString = atob(base64);
  return Uint8Array.from(binString, (m) => m.codePointAt(0));
}

export function decompressObject(base64) {
  const compressedBytes = base64ToBytes(base64);
  const bytes = inflate(compressedBytes);

  const jsonString = new TextDecoder().decode(bytes);

  const obj = JSON.parse(jsonString);
  return obj;
}

// RedstoneTorch Piston [isSticky=false] RedstoneRepeater RedstoneDust Lever Button Piston [isSticy=true] ObserverBlock WoolBlock [color]
let marcoBlocks = {
  Wool: "Dirt",
  RedstoneTorch: "RedstoneTorch",
  RedstoneDust: "RedstoneDust",
  Piston: "Piston",
  PistonHead: "PistonHead",
  RedstoneRepeater: "Repeater",
  RedstoneComparator: "Comparator",
  ObserverBlock: "Observer",
  Button: "Button",
  Lever: "Lever",
};

/*
direction: "Down"
isBeingPowered: false
isExtended: false
isSticky: false
movement: "None"
movementDirection: "Up"
type: "Piston"
*/
const snakeToPascal = (str) => str
    .split("_")
    .map((substr) => substr.charAt(0).toUpperCase() + substr.slice(1))
    .join("");


export function convertToMarcoBlocks(obj) {
  let blocklist = [];
  let save = SaveData.new_save();
  for (let [coord, blk] of Object.entries(obj)) {
    let [strx, stry] = coord.split(" ");
    let [x, y] = [parseInt(strx), parseInt(stry)];
    let marcoblock = marcoBlocks[blk.type] || "Dirt";
    let state = 0

    if (marcoblock === "Piston" && blk.isSticky) {
        marcoblock = "StickyPiston";
    } else if (marcoblock === "PistonHead") {
        marcoblock = ""; 
    } else if (marcoblock === "Dirt" && blk.color) {
        let color = blk.color;
        color = snakeToPascal(color);
        marcoblock = `${color}Wool`;
    } else if (marcoblock === "Repeater"){
        state = blk.ticksOn + blk.ticksOff
    } else if (marcoblock === "Comparator"){
        if (blk.mode === "add") {
            state = 0
        } else{
            state = 1
        }
    }

    if (marcoblock.length > 0) {
      blocklist.push(marcoblock);
      let orientation;
      if (blk.direction == "Up") {
        orientation = 0;
      } else if (blk.direction == "Right") {
        orientation = 1;
      } else if (blk.direction == "Down") {
        orientation = 2;
      } else if (blk.direction == "Left") {
        orientation = 3;
      }
      save.append_block(marcoblock, orientation, BigInt(-1 * y), BigInt(x), state);
    }
  }
  return save.json_string();
}
