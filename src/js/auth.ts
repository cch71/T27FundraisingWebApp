import * as AmazonCognitoIdentity from 'amazon-cognito-identity-js';
//import { Auth } from 'aws-amplify'

import config from '../config';


class CognitoAuth {
    private userPool: any | null = null;
    private authToken: any | null = null;

    constructor() {
        if (!(config.cognito.userPoolId &&
            config.cognito.userPoolClientId &&
            config.cognito.region))
        {
            console.error("Invalid Auth Config Data");
            return;
        }

        const poolData = {
            UserPoolId: config.cognito.userPoolId,
            ClientId: config.cognito.userPoolClientId
        };

        this.userPool = new AmazonCognitoIdentity.CognitoUserPool(poolData);

        this.authToken = new Promise((resolve: any, reject: any)=>{
            var cognitoUser = this.userPool.getCurrentUser();

            if (cognitoUser) {
                cognitoUser.getSession((err: any, session: any)=>{
                    if (err) {
                        reject(err);
                    } else if (!session.isValid()) {
                        resolve(null);
                    } else {
                        const token = session.getIdToken().getJwtToken();

                        // console.log("Getting User Pool Data");
                        // userPool.client.listUsers({
                        //     'UserPoolId': _config.cognito.userPoolId
                        // }, function(err, data) {
                        //     if (err) console.log(err, err.stack); // an error occurred
                        //     else {
                        //         console.log("User Data:");
                        //         console.log(data);           // successful response
                        //     }
                        // });

                        resolve(token);
                    }
                });
            } else {
                resolve(null);
            }
        });

    }

    currentUser(): any {
        return this.userPool.getCurrentUser();
    }

    validateSession(): Promise<any> {
        return new Promise((resolve) => {
            const cognitoUser = this.userPool.getCurrentUser();
 
            if (cognitoUser != null) {
                cognitoUser.getSession((err: any, session: any)=>{
                    if (err) {
                        resolve([false, cognitoUser.getUsername()]);
                    } else {
                        resolve([session.isValid(), cognitoUser.getUsername()]);
                    }
                });
            } else {
                resolve([false, null]);
            }
        });
    }

    signOut() {
        this.userPool.getCurrentUser().signOut();
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
            onSuccess: onSuccess,
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
