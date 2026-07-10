
/**
 * App-wide navigation helper. The shell and each dynamically loaded wasm
 * module run their own yew-router instance, and a router only notices its
 * own pushes plus browser popstate events. Navigating with pushState and
 * then dispatching a synthetic popstate wakes every router at once.
 * @param {string} path The absolute path to navigate to (e.g. "/reports")
 */
const navigateTo = (path) => {
    if (window.location.pathname !== path) {
        window.history.pushState({}, "", path);
    }
    window.dispatchEvent(new PopStateEvent("popstate"));
};

export { navigateTo };
