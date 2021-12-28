
/**
 *  Creates an auth object
 */
const authObj = new Auth0Client({
    domain: 'dev-pmq2q476.us.auth0.com',
    client_id: 'AewxkIRU4ckn5GcNH32qkTU1AYgdKKMn'
});

/**
 *  Retrieves user information
 */
const getUserInfo = async () => {
    const token = await authObj.getTokenSilently();
    const userInfo = await authObj.getUser();
    // console.log(`UserInfo: ${JSON.stringify(userInfo, null, '\t')}`);
    // console.log(`Token: ${JSON.stringify(token, null, '\t')}`);
    return {
        "email": userInfo.email,
        "token": token
    };
}

/**
 * Starts the authentication flow
 */
const loginUser = async (targetUrl) => {
    console.log("Starting login");

    const options = {
        redirect_uri: window.location.origin
    };

    if (targetUrl) {
        options.appState = { targetUrl };
    }

    await authObj.loginWithRedirect(options);
};

/**
 * Logs the user out
 */
const logoutUser = async (targetUrl) => {
    console.log("Starting logout");
    authObj.logout({
        returnTo: window.location.origin
    });
};

/**
 * Checks to see if the user is authenticated. If so, `fn` is executed. Otherwise, the user
 * is prompted to log in
 * @param {*} fn The function to execute if the user is logged in
 */
const isAuthenticated = async () => {
    let isAuthenticated = await authObj.isAuthenticated();

    if (isAuthenticated) {
        // show the gated content
        console.log(`JS1: Is Authenticated: True`);
        return;
    }

    // NEW - check for the code and state parameters
    const query = window.location.search;
    if (query.includes("code=") && query.includes("state=")) {

        // Process the login state
        await authObj.handleRedirectCallback();

        isAuthenticated = await authObj.isAuthenticated();

        if (isAuthenticated) {
            // show the gated content
            console.log(`JS2: Is Authenticated: True`);
            const token = await authObj.getTokenSilently();
            const userInfo = await authObj.getUser();
            console.log(`UserInfo: ${JSON.stringify(userInfo, null, '\t')}`);
            console.log(`Token: ${JSON.stringify(token, null, '\t')}`);
            const claims = await authObj.getIdTokenClaims();
            console.log(`Claims: ${JSON.stringify(claims, null, '\t')}`);
        }
        window.history.replaceState({}, document.title, "/");
    }
    return isAuthenticated;

};


export {loginUser, logoutUser, isAuthenticated, getUserInfo};
