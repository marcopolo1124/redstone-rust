<script>
    import { onMount } from 'svelte'
    import init, {run} from "redstone-wasm";
    import {decompressObject, convertToMarcoBlocks} from "$lib"
    let save = ""

    $: console.log({save})

    onMount(async() =>{
        console.log("mount")
        await init()
        run()
    })

    

</script>

<h1>hi</h1>
<form on:submit={() => {
    let obj = decompressObject(save)
    console.log({obj})
    convertToMarcoBlocks(obj)
}}>
    <textarea bind:value={save}></textarea>
    <input type="submit" value="save"/>
</form>
