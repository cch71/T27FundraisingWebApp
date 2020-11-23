import React from "react"
import {ListGroup, Card, Spinner} from 'react-bootstrap';
import NavBar from "../components/navbar"
import auth from "../js/auth"
import { navigate } from "gatsby"
import {orderDb} from "../js/ordersdb"
import {FundraiserConfig, downloadFundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"
import awsConfig from "../config"



export default function home() {

    let notReadyView = 'col-xs-1 d-flex justify-content-center';
    let readyView = {display: 'none'};

    // If no active user go to login screen
    auth.getSession().then((results)=>{
        const [isValidSession, session] = results;
        if (!isValidSession) {
            navigate('/signon/');
            return;
        }
        console.log(`Active User: ${auth.currentUserEmail()}`);

        const authToken = session.getIdToken().getJwtToken();
        //queryOrders(authToken);

        
        const enableReady = ()=>{
            const readyViewElm = document.getElementById('readyView');
            if (readyViewElm) {
                readyViewElm.style.display = "block";
            } else {
                readyView = {display: 'block'};
            }

            const notReadyViewElm = document.getElementById('notReadyView');
            if (notReadyViewElm) {
                notReadyViewElm.className = "d-none";
            } else {
                notReadyView = 'd-none';
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


    const addNewOrder = ()=>{
        console.log("Add new order");
        navigate('/order_step_1/');
    };

    const summary = orderDb.getOrderSummary();

    return (
        <div>
            <div id="notReadyView" className={notReadyView} >
                <Spinner animation="border" role="status">
                    <span className="sr-only">Loading...</span>
                </Spinner>
            </div>
            <div id="readyView" style={readyView}>
                <NavBar/>
                <div className="col-xs-1 d-flex justify-content-center">
                    <Card>
                        <Card.Body>
                            <Card.Title>Summary Information</Card.Title>
                            <div>You have {summary.numOrders} orders.</div>
                            <div>You have collected {summary.totalOrderCost}</div>
                            <div>Of that {summary.totalDonations} are donations</div>
                            <div>{summary.totalOrders} is from product</div>
                        </Card.Body>
                    </Card>
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
