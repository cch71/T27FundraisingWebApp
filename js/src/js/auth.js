
import Keycloak from "https://cdn.jsdelivr.net/npm/keycloak-js@26.0.7/lib/keycloak.min.js"

const initOptions = {
    url: 'https://usw2.auth.ac/auth', realm: 't27fr', clientId: 't27frapp', onLoad: 'login-required'
}

/**
 *  Creates an auth object
 */
const keycloak = new Keycloak(initOptions);



/**
 *  Retrieves user information
 */
const getUserInfo = async () => {
    const parsedToken = keycloak.idTokenParsed;
    const token = keycloak.idToken;
    // console.log(`UserInfo: ${JSON.stringify(parsedToken, null, '\t')}`);
    // console.log(`Token: ${JSON.stringify(token, null, '\t')}`);
    let roles = parsedToken.groups.map((role) => {
        return role.startsWith("/") ? role.substring(1) : role;
    });
    const resp = {
        "email": parsedToken.email,
        "token": token,
        "roles": roles,
        "id": parsedToken.preferred_username,
        "name": parsedToken.name
    };

    return resp;
}

/**
 * Starts the authentication flow
 */
const loginUser = async () => {
    // console.log("Starting login");

    const auth = await keycloak.init({ onLoad: initOptions.onLoad });

    if (!auth) {
        window.location.reload();
    } else {
        console.info("Authenticated");
    }


    //Token Refresh
    setInterval(() => {
        keycloak.updateToken(70).then((refreshed) => {
            if (refreshed) {
                console.info('Token refreshed' + refreshed);
                // } else {
                //     console.warn('Token not refreshed, valid for '
                //                  + Math.round(keycloak.tokenParsed.exp
                //                               + keycloak.timeSkew - new Date().getTime() / 1000) + ' seconds');
            }
        }).catch(() => {
            console.error('Failed to refresh token');
            logoutUser()
        });
    }, 6000)
};

/**
 * Logs the user out
 */
const logoutUser = async () => {
    // console.log("Starting logout");
    keycloak.logout({
        redirectUri: window.location.origin
    });
};

/**
 * Checks to see if the user is authenticated. If so, `fn` is executed. Otherwise, the user
 * is prompted to log in
 * @param {*} fn The function to execute if the user is logged in
 */
const isAuthenticated = async () => {
    try {
        await loginUser();
        return true;
    } catch {
        return false;
    }
};


export { loginUser, logoutUser, isAuthenticated, getUserInfo };
