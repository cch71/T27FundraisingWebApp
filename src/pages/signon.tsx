import React from "react"
import NavBar from "../components/navbar"
import auth from "../js/auth"
import { navigate } from "gatsby"
import {orderDb} from "../js/ordersdb"


export default function signon() {
    //If you got here then lets go ahean and sign out if already signed in.
    auth.signOut();
    orderDb.setActiveOrder(); // Reset active order


    const onFormSubmission = (event: any) => {
        const form = event.currentTarget;

        const loginId = form[0].value;
        const pw = form[1].value;
        //console.log(`Form submttted uid: ${loginId}`)

        event.preventDefault();
        event.stopPropagation();

        const onSuccess=(autoInfo: any)=>{
            console.log(autoInfo);
            navigate('/'/*, {replace: true} see if disabling fixes spinning menu*/);
        };
        const onFailure=()=>{
            console.error("authenticaiton Error");
            setValidated(false);
        };
        auth.signIn(loginId, pw, onSuccess, onFailure);
    };

    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <div className="card" style={{ width: '20rem' }}>
                <div className="card-body">
                    <h5 className="card-title">Sign On</h5>
                    <form onSubmit={onFormSubmission}>
                        <div className="row mb-3">
                            <label htmlFor="formLoginId">Login ID</label>
                            <input type="text" className="form-control" id="formLoginId"
                                   aria-describedby="loginIdHelp" placeholder="Enter Login ID" />
                            <small id="loginIdHelp" className="form-text text-muted">
                                For questions contact the Fundraising Coordinator
                            </small>
                        </div>
                        <div className="row mb-3">
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
