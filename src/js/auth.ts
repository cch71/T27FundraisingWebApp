import * as AmazonCognitoIdentity from 'amazon-cognito-identity-js';
//import { Auth } from 'aws-amplify'

import config from '../config';

class CognitoAuth {
    private userPool: any | undefined;
    private isUserAuthenticated: boolean = false;

    constructor() {
        if (!(config.cognito.userPoolId &&
            config.cognito.userPoolClientId &&
            config.cognito.region))
        {
            throw new Error("Invalid Auth Config Data");
        }

        const poolData = {
            UserPoolId: config.cognito.userPoolId,
            ClientId: config.cognito.userPoolClientId
        };

        this.userPool = new AmazonCognitoIdentity.CognitoUserPool(poolData);

        // this.authToken = new Promise((resolve: any, reject: any)=>{
        //     var cognitoUser = this.userPool.getCurrentUser();

        //     if (cognitoUser) {
        //         cognitoUser.getSession((err: any, session: any)=>{
        //             if (err) {
        //                 reject(err);
        //             } else if (!session.isValid()) {
        //                 resolve(null);
        //             } else {
        //                 const token = session.getIdToken().getJwtToken();

        //                 // console.log("Getting User Pool Data");
        //                 // userPool.client.listUsers({
        //                 //     'UserPoolId': _config.cognito.userPoolId
        //                 // }, function(err, data) {
        //                 //     if (err) console.log(err, err.stack); // an error occurred
        //                 //     else {
        //                 //         console.log("User Data:");
        //                 //         console.log(data);           // successful response
        //                 //     }
        //                 // });

        //                 resolve(token);
        //             }
        //         });
        //     } else {
        //         resolve(null);
        //     }
        // });

    }

    currentUser(): any {
        return this.userPool.getCurrentUser();
    }

    getCurrentUserId(): string {
        return this.currentUser().getUsername().replace('-at-', '@')
    }

    async getAuthToken(): string {
        const [isValid, session] = await this.getSession();
        if (!(isValid && session)) {
            throw(new Error("Invalid Session"));
        }
        const idToken = session.getIdToken();
        const payload = idToken.decodePayload()
        //console.log(`CU: ${JSON.stringify(payload, null, '\t')}`)
        return(idToken.getJwtToken());
    }

    async getUserIdAndGroups() : [string, string] {
        const [isValid, session] = await this.getSession();
        if (!(isValid && session)) {
            throw(new Error("Invalid Session"));
        }
        const cognitoUser = this.userPool.getCurrentUser().getUsername();
        const idPayload = session.getIdToken().decodePayload();
        return [cognitoUser, idPayload['cognito:groups']]
    }

    getSession(): Promise<any> {
        return new Promise((resolve, reject) => {
            const cognitoUser = this.userPool.getCurrentUser();

            if (!cognitoUser) {
                resolve([false, null]);
                return;
            }

            try {
                cognitoUser.getSession((err: any, session: any)=>{
                    if (err) {
                        resolve([false, session]);
                    } else {
                        this.isUserAuthenticated = session.isValid;
                        resolve([session.isValid(), session]);
                    }
                });
            } catch(err) {
                reject(err);
            }
        });
    }

    signOut() {
        this.isUserAuthenticated = false;
        if (this.userPool) {
            const curUser = this.currentUser();
            if (curUser) {
                curUser.signOut();
            }
        }
    }

    private normalizeLoginId(loginId: string): string {
        return loginId.replace('@', '-at-');
    }


    signIn(loginId: string, pw: string, onSuccess: any, onFailure: any) {
        const authenticationDetails = new AmazonCognitoIdentity.AuthenticationDetails({
            Username: this.normalizeLoginId(loginId),
            Password: pw
        });

        const cognitoUser = this.createCognitoUser(loginId);
        cognitoUser.authenticateUser(authenticationDetails, {
            onSuccess: (authInfo: any)=>{
                this.isUserAuthenticated = true;
                if (onSuccess) { onSuccess(authInfo); }
            },
            onFailure: onFailure
        });
    }

    // verify(loginId: string, code: string, onSuccess: any, onFailure: any) {
    //     createCognitoUser(loginId).confirmRegistration(
    //         code,
    //         true,
    //         function confirmCallback(err, result) {
    //             if (!err) {
    //                 onSuccess(result);
    //             } else {
    //                 onFailure(err);
    //             }
    //         });
    // }

    private createCognitoUser(loginId: string): any {
        return new AmazonCognitoIdentity.CognitoUser({
            Username: this.normalizeLoginId(loginId),
            Pool: this.userPool
        });
    }

}

const auth = new CognitoAuth();
export default auth;
