import React from "react"
import {ListGroup, Card} from 'react-bootstrap';
import NavBar from "../components/navbar"
import Config from "../config"
import auth from "../js/auth"
import { navigate } from "gatsby"
import {orderDb} from "../js/ordersdb"


export default function home() {

    // If no active user go to login screen
    auth.validateSession().then((results)=>{
        const [isValidSession, username] = results; 
        if (!isValidSession) {
            if (username) {
                console.log(`!!!!!!! Non Active User: ${username}`);
            } else {
                console.log(`No User Found`);
            }

            navigate('/signon', {replace: true});
            return;
        }
        console.log(`Active User: ${username}`)
    });
    
    
    const addNewOrder = ()=>{
        console.log("Add new order");
        navigate('/order_step_1', {replace: true});
    };

    const summary = orderDb.getOrderSummary();
    
    return (
        <div>
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
    );
}
