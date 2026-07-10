
/**
 * Loads the page-module wasm binaries on demand. Each module is built with
 * `wasm-bindgen --target web` into /modules/<name>/ (see build_modules.sh)
 * and exports `mount(rootId)` / `unmount()`. A module instance (and any
 * state held in its wasm memory) is kept for the life of the page, so
 * revisiting a module doesn't re-download or re-initialize it.
 */
const loadedModules = {};

// Guards against a stale mount when the user switches modules while a
// download is still in flight: only the most recent load may mount.
let mountEpoch = 0;

const loadModule = async (name, rootId) => {
    const epoch = ++mountEpoch;

    let mod = loadedModules[name];
    if (!mod) {
        mod = await import(`/modules/${name}/${name}.js`);
        // Initialize the wasm binary; its URL resolves relative to the JS glue
        await mod.default();
        loadedModules[name] = mod;
    }

    if (epoch !== mountEpoch) {
        return;
    }
    await mod.mount(rootId);
};

const unloadModule = (name) => {
    mountEpoch += 1;
    loadedModules[name]?.unmount();
};

export { loadModule, unloadModule };
