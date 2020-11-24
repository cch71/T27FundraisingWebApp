import React, { useState, useEffect } from "react"
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

    // Client-side Runtime Data Fetching
    const [orderList, setOrderList] = useState();
    useEffect(() => {
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
        orderDb.query(fieldNames).then((orders: Array<any>)=>{
            console.log(`Orders Page: ${JSON.stringify(orders)}`);
            const orderElmList = [];
            for (const order of orders) {
                const nameStr = `${order.firstName}, ${order.lastName}`;
                const addrStr = `${order.addr1} ${order.addr2} ${order.city} ${order.usStateAbbr} ${order.zip}`;
                const totalDueStr = USD(order.totalDue).format();
                orderElmList.push(
                    <ul className="list-group list-group-horizontal-lg my-2" key={order.orderId}>
                        <li className="list-group-item order-list-name">${nameStr}</li>
                        <li className="list-group-item order-list-addr">${addrStr}</li>
                        <li className="list-group-item order-list-due">${totalDueStr}</li>
                        <li className="list-group-item">
                            <button type="button" className="btn btn-danger mx-1 float-right">X</button>
                            <button type="button" className="btn btn-info float-right">I</button>
                        </li>
                    </ul>
                );
            }
            setOrderList(orderElmList);
        });
    }, [])

    return (
        <div>
            <NavBar/>
            <button type="button"
                    className="btn btn-outline-light add-order-btn"
                    onClick={addNewOrder}>
                +
            </button>
            <div className="col-xs-1 d-flex justify-content-center">
                <div className="card">
                    <div className="card-body">
                        <h5 className="card-title" id="orderCardTitle">Orders</h5>
                        {orderList}
                    </div>
                </div>
            </div>
        </div>
    );
}
