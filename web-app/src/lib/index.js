// place files you want to import through the `$lib` alias in this folder.
import {inflate} from "pako";
import { SaveData } from "redstone-wasm";

function base64ToBytes (base64) {
    const binString = atob(base64)
    return Uint8Array.from(binString, m => m.codePointAt(0))
}

export function decompressObject (base64) {
    const compressedBytes = base64ToBytes(base64)
    const bytes = inflate(compressedBytes)

    const jsonString = new TextDecoder().decode(bytes)

    const obj = JSON.parse(jsonString)
    return obj
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
}

/*
direction: "Down"
isBeingPowered: false
isExtended: false
isSticky: false
movement: "None"
movementDirection: "Up"
type: "Piston"
*/

export function convertToMarcoBlocks(obj) {
    let blocklist = []
    for (let [coord, blk] of Object.entries(obj)){
        let [strx, stry] = coord.split(" ")
        let [x, y] = [parseInt(strx), parseInt(stry)]

        console.log({strx, stry, x, y})
        let marcoblock = marcoBlocks[blk.type] || "Dirt"
        if (marcoblock === "Piston" && blk.isSticky){
            marcoblock = "StickyPiston"
        }
        if (marcoblock === "PistonHead"){
            marcoblock = ""
        }
        if (marcoblock === "Dirt" && blk.color){
            let color = blk.color
            color = `${color[0].toUpperCase()}${color.slice(1)}`
            marcoblock = `${color}Wool`
        }

        if (marcoblock.length > 0){
            blocklist.push(marcoblock)
            let orientation
            if (blk.direction == "Up"){
                orientation = 0
            } else if (blk.direction == "Right"){
                orientation = 1
            } else if (blk.direction == "Down"){
                orientation = 2
            } else if (blk.direction == "Left"){
                orientation = 3
            }

            let save = SaveData.new()
            //append(blk_type: string, orientation: number, x: bigint, y: bigint)
            save.append(marcoblock, orientation, BigInt(x), BigInt(y))
            console.log({save: save.json_string()})
        }
    }
    console.log({blocklist})
}