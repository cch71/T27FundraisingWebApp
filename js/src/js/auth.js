
import { UserManager, WebStorageStateStore } from "oidc-client-ts";

// Provider settings live in webapp/index.html (window.authConfig) so switching
// OAuth2/OIDC providers (AWS Cognito, Keycloak, Auth0, ...) is a config-only change.
const authConfig = window.authConfig;

const userManager = new UserManager({
    authority: authConfig.authority,
    client_id: authConfig.clientId,
    redirect_uri: authConfig.redirectUri || window.location.origin,
    post_logout_redirect_uri: window.location.origin,
    response_type: "code",
    scope: authConfig.scope || "openid email profile",
    automaticSilentRenew: true,
    // Keep tokens and transient auth state in sessionStorage rather than
    // localStorage. sessionStorage is scoped to the tab and cleared when the
    // tab closes, so the access/refresh tokens and the PKCE code_verifier are
    // never persisted to long-lived, cross-tab storage that any XSS on the
    // origin (or a compromised dependency) could scrape. sessionStorage still
    // survives the login redirect within the same tab, so the code flow works.
    userStore: new WebStorageStateStore({ store: window.sessionStorage }),
    stateStore: new WebStorageStateStore({ store: window.sessionStorage }),
});

// In-memory copy of the signed-in user. The app clears web storage on logoff
// before calling logoutUser(), so the id_token_hint must not depend on storage.
let currentUser = null;

userManager.events.addUserLoaded((user) => {
    currentUser = user;
});

userManager.events.addSilentRenewError((err) => {
    console.error("Failed to refresh token", err);
    logoutUser();
});

// Claims that hold group/role membership, by provider convention:
// Keycloak: "groups" (may be "/" prefixed), AWS Cognito: "cognito:groups"
const ROLES_CLAIMS = ["groups", "cognito:groups", "roles"];

// Claims that hold the username:
// Keycloak: "preferred_username", AWS Cognito: "cognito:username"
const ID_CLAIMS = ["preferred_username", "cognito:username", "username"];

const firstClaim = (profile, configured, candidates) => {
    for (const claim of configured ? [configured, ...candidates] : candidates) {
        if (profile[claim] !== undefined) {
            return profile[claim];
        }
    }
    return undefined;
};

/**
 *  Retrieves user information
 */
const getUserInfo = async () => {
    const user = currentUser || (await userManager.getUser());
    const profile = user.profile;
    const rawRoles = firstClaim(profile, authConfig.rolesClaim, ROLES_CLAIMS) || [];
    const roles = rawRoles.map((role) => {
        return role.startsWith("/") ? role.substring(1) : role;
    });
    return {
        "email": profile.email,
        "token": authConfig.tokenType === "access" ? user.access_token : user.id_token,
        "roles": roles,
        "id": firstClaim(profile, authConfig.idClaim, ID_CLAIMS),
        "name": profile.name,
    };
};

/**
 * Starts the authentication flow. Redirects to the provider's login page if
 * there is no valid session, so this may never resolve.
 */
const loginUser = async () => {
    // console.log("Starting login");
    const params = new URLSearchParams(window.location.search);

    if (params.has("error")) {
        throw new Error(`Login failed: ${params.get("error_description") || params.get("error")}`);
    }

    if (params.has("code") && params.has("state")) {
        // Returning from the provider's login page
        currentUser = await userManager.signinRedirectCallback();
        window.history.replaceState({}, document.title, window.location.pathname);
        console.info("Authenticated");
        return;
    }

    const user = await userManager.getUser();
    if (user && !user.expired) {
        currentUser = user;
        console.info("Authenticated");
        return;
    }

    if (user && user.refresh_token) {
        try {
            currentUser = await userManager.signinSilent();
            console.info("Authenticated (session renewed)");
            return;
        } catch (err) {
            console.warn("Silent renew failed, re-authenticating", err);
        }
    }

    await userManager.signinRedirect();
    // Navigating away to the provider's login page; block until unload
    await new Promise(() => {});
};

/**
 * Logs the user out
 */
const logoutUser = async () => {
    // console.log("Starting logout");
    const user = currentUser;
    currentUser = null;
    await userManager.removeUser();

    if (authConfig.logoutEndpoint) {
        // Providers without RP-initiated logout support (e.g. AWS Cognito
        // hosted UI uses /logout?client_id=...&logout_uri=...)
        const url = new URL(authConfig.logoutEndpoint);
        url.searchParams.set("client_id", authConfig.clientId);
        url.searchParams.set("logout_uri", window.location.origin);
        window.location.assign(url);
    } else {
        await userManager.signoutRedirect({
            id_token_hint: user?.id_token,
            post_logout_redirect_uri: window.location.origin,
        });
    }
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
