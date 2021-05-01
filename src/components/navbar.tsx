import React, { useState, useEffect } from 'react'
import { Link, navigate } from 'gatsby'
import auth from "../js/auth"
import awsConfig from "../config"
import t27patch from "../../static/t27patch.png"
import {orderDb} from "../js/ordersdb"
import {saveCurrentOrder} from "../js/utils"

////////////////////////////////////////////////////////////////////
//
const newIssueDlg = (event: any)=>{

    const sendIssueToJira = async (issue: any)=> {
        const authToken = await auth.getAuthToken();
        const resp = await fetch(awsConfig.api.invokeUrl + '/fileissue', {
            method: 'post',
            body: JSON.stringify(issue),
            headers: {
                'Content-Type': 'application/json',
                Authorization: authToken
            }
        });

        if (!resp.ok) { // if HTTP-status is 200-299
            const errRespBody = await resp.text();
            throw new Error(`File Issue Req error: ${resp.status}  ${errRespBody}`);
        } else {
            console.log(`Issue Filed: ${await resp.text()}`);// await resp.json();
        }
    };

    const onXmitIssue = (event: any)=> {
        const sumElm = document.getElementById('formSummary');
        const descElm = document.getElementById('formDescription');

        let isAllGood = true
        if (!sumElm.value) {
            sumElm.classList.add('is-invalid');
            isAllGood = false;
        } else {
            sumElm.classList.remove('is-invalid');
        }

        if (!descElm.value) {
            descElm.classList.add('is-invalid');
            isAllGood = false;
        } else {
            descElm.classList.remove('is-invalid');
        }

        if (!isAllGood) { return; }

        const jiraIssue = {
            summary: sumElm.value,
            description: descElm.value
        };
        const submitBtnElm = event.currentTarget;

        submitBtnElm.disabled = true;
        jQuery('#formXmitIssueSpinner').show();
        sendIssueToJira(jiraIssue)
            .then(()=>{
                submitBtnElm.disabled = false;
                jQuery('#formXmitIssueSpinner').hide();
                document.getElementById('formXmitIssue').reset();
                bootstrap.Modal.getInstance(document.getElementById('xmitIssueDlg')).hide();
            })
            .catch((err)=>{
                submitBtnElm.disabled = false;
                jQuery('#formXmitIssueSpinner').hide();
                console.error(err);
            });
    };

    return(
        <div className="modal fade" id="xmitIssueDlg"
             tabIndex="-1" role="dialog" aria-labelledby="xmitIssueDlgTitle" aria-hidden="true">
            <div className="modal-dialog modal-dialog-centered" role="document">
                <div className="modal-content">
                    <div className="modal-header">
                        <h5 className="modal-title" id="xmitIssueDlgLongTitle">
                            Submit New Issue with Fundraiser App
                        </h5>
                        <button type="button" className="close" data-bs-dismiss="modal" aria-label="Close">
                            <span aria-hidden="true">&times;</span>
                        </button>
                    </div>
                    <div className="modal-body">
                        <form className="needs-validation" id="formXmitIssue" noValidate>
                            <div className="row mb-2 g-2">
                                <div className="form-floating">
                                    <input className="form-control" type="text" autoComplete="fr-new-issue" id="formSummary"
                                           placeholder="Summary" required maxLength="255"/>
                                    <label htmlFor="formSummary">
                                        Summary (255 Mex Chars)
                                    </label>
                                </div>
                            </div>
                            <div className="row mb-2 g-2">
                                <div className="form-floating">
                                    <textarea className="form-control" rows="10" required id="formDescription"/>
                                    <label htmlFor="formDescription">Description of problem</label>
                                </div>
                            </div>
                        </form>
                    </div>
                    <div className="modal-footer">
                        <button type="button" className="btn btn-secondary" data-bs-dismiss="modal">Cancel</button>
                        <button type="button" className="btn btn-primary" onClick={onXmitIssue}>
                            <span className="spinner-border spinner-border-sm me-1" role="status"
                                  aria-hidden="true" id="formXmitIssueSpinner" style={{display: "none"}} />
                            Submit
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};

////////////////////////////////////////////////////////////////////
//
const NavBar = (props) => {
    const activePathNm = (typeof window !== 'undefined')?window.location.pathname:undefined;
    console.log(`Path Name ${activePathNm}`);

    const setIfActive = (srchPath: string) => {
        if (activePathNm===srchPath || `${activePathNm}/`===srchPath) {
            return('nav-item nav-link active');
        } else {
            return('nav-item nav-link');
        }
    };

    const collapseNav = () => {
        const srchPath = '/order_step_1/';
        if (activePathNm===srchPath || `${activePathNm}/`===srchPath) {
            saveCurrentOrder(); //If we navigate away lets save current order if it is active
        }
        jQuery(".navbar-collapse").collapse('hide');
    }

    const handleSignout = ()=>{
        collapseNav();
        auth.signOut();
        if ("/" === activePathNm) {
            window.location.reload(false);
        } else {
            navigate("/");
        }
    };

    const onXmitIssue = ()=>{
        collapseNav();
    };


    const [userName, setUserName] = useState();
    const [adminMenuItems, setAdminMenuItems] = useState();
    useEffect(() => {
        const onAsyncView = async ()=>{
            //If this throws then we aren't authenticated so don't show bar anyways
            const [uid, _] = await auth.getUserIdAndGroups();

            if (await auth.isCurrentUserAdmin()) {
				setAdminMenuItems(
					<div className='dropdown-item' onClick={ ()=>{navigate("/delivery_time_sheet")} }>Delivery TimeSheet</div>
				);
			}

            setUserName(uid);
        };

        onAsyncView()
            .then()
            .catch((err)=>{});
    }, []);

    return (
        <>
            <nav className="navbar sticky-top navbar-expand-sm navbar-light bg-light" id="primaryNavBar">
                <a className="navbar-brand" href="#">
                    <span>
                        <img className="navbar-logo ms-2" src={t27patch} alt="Logo" />
                    </span>
                </a>

                <button className="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav"
                        aria-controls="navbarNav" aria-expanded="false" aria-label="Toggle navigation">
                    <span className="navbar-toggler-icon"></span>
                </button>

                <div className="collapse navbar-collapse" id="navbarNav">
                    <ul className="navbar-nav me-auto">
                        <li>
                            <Link className={setIfActive('/')} replace to='/' onClick={collapseNav}>Home</Link>
                        </li>
                        <li>
                            <Link className={setIfActive('/reports/')} replace to='/reports/' onClick={collapseNav}>
                                Reports
                            </Link>
                        </li>
                        <li style={{display: (orderDb.getActiveOrder()?'block':'none')}} >
                            <Link className={setIfActive('/order_step_1/')} replace to='/order_step_1/' onClick={collapseNav}>
                                Open Order
                            </Link>
                        </li>
                        <li>
                            <a className='nav-item nav-link' href='https://cch71.github.io/T27FundraisingWebApp/'
                               target="_blank" onClick={collapseNav}>
                                Help
                            </a>
                        </li>

                    </ul>
                    <span className="navbar-nav nav-item dropdown">
                        <a className="nav-link dropdown-toggle" href="#" id="navbarDropdown"
                           data-bs-toggle="dropdown" aria-expanded="false" role="button">
                            {userName}
                        </a>
                        <div className="dropdown-menu dropdown-menu-end" aria-labelledby="navbarDropdown">
                            <div className='dropdown-item' onClick={handleSignout}>Signout</div>
							{adminMenuItems}
                            <a className='dropdown-item'
                               href="#xmitIssueDlg" data-bs-toggle="modal" onClick={onXmitIssue}>Report Issue</a>
                        </div>
                    </span>
                </div>
            </nav>
            {newIssueDlg()}
        </>
    );
}

export default NavBar
