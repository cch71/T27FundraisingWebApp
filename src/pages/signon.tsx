import React from "react"
import NavBar from "../components/navbar"
import auth from "../js/auth"
import { navigate } from "gatsby"

export default function signon() {
    const onFormSubmission = (event: any) => {
        const form = event.currentTarget;

        const loginId = form[0].value;
        const pw = form[1].value;
        console.log(`Form submttted uid: pw: ${loginId} ${pw}`)

        event.preventDefault();
        event.stopPropagation();

        const onSuccess=(autoInfo: any)=>{
            console.log(autoInfo);
            navigate('/', {replace: true});
        };
        const onFailure=()=>{
            console.error("authenticaiton Error");
            setValidated(false);
        };
        auth.signIn(loginId, pw, onSuccess, onFailure);
    };

    const curUsr = auth.currentUser();
    const loginId = (curUsr && curUsr.username) ? curUsr.username : '';


    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <div className="card" style={{ width: '20rem' }}>
                <div className="card-body">
                    <h5 className="card-title">Sign On</h5>
                    <form onSubmit={onFormSubmission}>
                        <div className="form-group">
                            <label htmlFor="formLoginId">Login ID</label>
                            <input type="text" className="form-control" id="formLoginId" defaultValue={loginId}
                                   aria-describedby="loginIdHelp" placeholder="Enter Login ID" />
                            <small id="loginIdHelp" className="form-text text-muted">
                                If you don't know what yours is check with TODO: Email
                            </small>
                        </div>
                        <div className="form-group">
                            <label htmlFor="formPassword">Password</label>
                            <input type="password" className="form-control" id="formPassword" placeholder="Password" />
                        </div>
                        <button type="submit" className="btn btn-primary">Submit</button>
                    </form>
                </div>
            </div>
        </div>
    );
}
