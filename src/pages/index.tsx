import React, { useState, useEffect } from "react"
import NavBar from "../components/navbar"
import auth from "../js/auth"
import { navigate } from "gatsby"
import {orderDb} from "../js/ordersdb"
import {FundraiserConfig, downloadFundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"
import awsConfig from "../config"



export default function home() {

    // If no active user go to login screen
    useEffect(() => {
        auth.getSession().then((results)=>{
            const [isValidSession, session] = results;
            if (!isValidSession) {
                navigate('/signon/');
                return;
            }
            console.log(`Active User: ${auth.currentUserEmail()}`);

            const authToken = session.getIdToken().getJwtToken();
            
            const enableReady = ()=>{
                const readyViewElm = document.getElementById('readyView');
                if (readyViewElm) {
                    readyViewElm.style.display = "block";
                }

                const notReadyViewElm = document.getElementById('notReadyView');
                if (notReadyViewElm) {
                    notReadyViewElm.className = "d-none";
                }
            };

            try {
                getFundraiserConfig();
                enableReady();
            } catch(err: any) {
                try {
                    downloadFundraiserConfig(authToken).then((loadedConfig: FundraiserConfig | null)=>{
                        if (null===loadedConfig) {
                            alert("Failed to load session fundraising config");
                        }
                        enableReady();
                    });
                } catch(err: any) {
                    alert("Failed: " + err);
                }
            }
        });
    }, []);


    const addNewOrder = ()=>{
        console.log("Add new order");
        navigate('/order_step_1/');
    };
    
    const summary = orderDb.getOrderSummary();

    return (
        <div>
            <div id="notReadyView" className='col-xs-1 d-flex justify-content-center' >
                <div className="spinner-border" role="status">
                    <span className="sr-only">Loading...</span>
                </div>
            </div>
            <div id="readyView" style={{display: 'none'}}>
                <NavBar/>
                <div className="col-xs-1 d-flex justify-content-center">
                    <div className="card">
                        <div className="card-body">
                            <h5 className="card-title">Summary Information</h5>
                            <div>You have {summary.numOrders} orders.</div>
                            <div>You have collected {summary.totalOrderCost}</div>
                            <div>Of that {summary.totalDonations} are donations</div>
                            <div>{summary.totalOrders} is from product</div>
                        </div>
                    </div>
                    <button type="button"
                            className="btn btn-outline-light add-order-btn"
                            onClick={addNewOrder}>
                        +
                    </button>
                </div>
            </div>
        </div>
    );
}
