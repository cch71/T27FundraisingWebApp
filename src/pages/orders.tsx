import React from "react"
import {ListGroup, Card, Button} from "react-bootstrap"
import NavBar from "../components/navbar"
import { navigate } from "gatsby"
import {orderDb, OrderIf} from "../js/ordersdb"
import currency from "currency.js"



export default function orders() {
    const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });
    
    const addNewOrder=()=>{
        console.log("Add new order");
        navigate('/order_step_1/');
    };
    const editOrder=(order: OrderIf)=>{
        console.log(`Editorder ${order.orderId}`);
    };

    const fieldNames:Array<string> = [
        "orderId",
        "firstName",
        "lastName",
        "addr1",
        "addr2",
        "city",
        "usStateAbbr",
        "zip",
        "totalDue"
    ];
    //"addr2", ${order.addr2}

    const orderElements:any = [];
    orderDb.query(fieldNames).then((orders: Array<any>)=>{
        console.log(`Orders Page: ${JSON.stringify(orders)}`);
        for (const order of orders) {

            const nameStr = `${order.firstName}, ${order.lastName}`;
            const addrStr = `${order.addr1} ${order.addr2} ${order.city} ${order.usStateAbbr} ${order.zip}`;
            const totalDueStr = USD(order.totalDue).format();
            
            //const li = clone.querySelectorAll("li");
            //li[0].textContent = 
            const htmlStr = `
                <ul class="list-group list-group-horizontal-lg my-2">
                    <li class="list-group-item order-list-name">${nameStr}</li>
                    <li class="list-group-item order-list-addr">${addrStr}</li>
                    <li class="list-group-item order-list-due">${totalDueStr}</li>
                    <li class="list-group-item">
                        <button type="button" class="btn btn-danger mx-1 float-right">X</button>
                        <button type="button" class="btn btn-info float-right">I</button>
                    </li>
                </ul>
            `;
            
            const ctId = document.getElementById("orderCardTitle");
            if (null!==ctId) { ctId.insertAdjacentHTML("afterend",htmlStr); }
        }
    });


    return (
        <div>
            <NavBar/>
            <button type="button"
                    className="btn btn-outline-light add-order-btn"
                    onClick={addNewOrder}>
                +
            </button>
            <div className="col-xs-1 d-flex justify-content-center">
                <Card>
                    <Card.Body id="orderCardBody">
                        <Card.Title id="orderCardTitle">Orders</Card.Title>
                    </Card.Body>
                </Card>
            </div>

            <template id="orderRow">
            </template>
        </div>
);
}
