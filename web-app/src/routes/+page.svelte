<script>
    import { onMount } from 'svelte'
    import init, {run} from "redstone-wasm";
    import {decompressObject, convertToMarcoBlocks} from "$lib"
    import { invalidate } from '$app/navigation';
    let save = ""

    onMount(async() =>{
        await init()
        run()
    })

</script>

<form on:submit={() => {
    let obj = decompressObject(save)
    let json_string = convertToMarcoBlocks(obj)
    localStorage.setItem("save/save_data.json", json_string)
    location.reload()
}}>
    <textarea bind:value={save}></textarea>
    <input type="submit" value="save"/>
</form>
