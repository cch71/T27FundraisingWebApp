import React, { useState, useEffect } from "react"
import NavBar from "../components/navbar"
import { navigate } from "gatsby"
import {orderDb, OrderListItem} from "../js/ordersdb"
import currency from "currency.js"



export default function orders() {
    const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });
    
    const addNewOrder=()=>{
        console.log("Add new order");
        orderDb.newActiveOrder();
        navigate('/order_step_1/');
    };

    // Client-side Runtime Data Fetching
    const [orderList, setOrderList] = useState();
    useEffect(() => {
        const onDeleteOrder = (event: any)=>{
            const btn = event.currentTarget;
            console.log(`Deleting order for ${btn.dataset.orderid}`);
        };

        const onEditOrder = (event: any)=>{
            const btn = event.currentTarget;
            const orderId = btn.dataset.orderid;
            console.log(`Editing order for ${orderId}`);
            orderDb.setActiveOrder(); // Reset active order to let order edit for set it
            navigate('/order_step_1/', {state: {editOrderId: orderId}});
        };

        orderDb.getOrderList().then((orders: Array<OrderListItem<string>>)=>{
            console.log(`Orders Page: ${JSON.stringify(orders)}`);
            const orderElmList = [];
            for (const order of orders) {
                order.email = order.email?order.email:'';
                order.addr2 = order.addr2?order.addr2:'';
                const nameStr = `${order.firstName}, ${order.lastName}`;
                const contactInfoStr = `${order.addr1} ${order.addr2} ${order.neighborhood} ${order.phone} ${order.email}`;
                const totalAmountStr = USD(order.amountTotal).format();
                orderElmList.push(
                    <ul className="list-group list-group-horizontal-lg my-2" key={order.orderId}>
                        <li className="list-group-item order-list-name">{nameStr}</li>
                        <li className="list-group-item order-list-addr">{contactInfoStr}</li>
                        <li className="list-group-item order-list-due">{totalAmountStr}</li>
                        <li className="list-group-item">
                            <button type="button" className="btn btn-outline-danger  mx-1 float-right order-edt-btn"
                                    data-orderid={order.orderId} onClick={onDeleteOrder}>
                                X
                            </button>
                            <button type="button" className="btn btn-outline-info float-right order-edt-btn"
                                    data-orderid={order.orderId} onClick={onEditOrder}>
                                <span>&#9999;</span>
                            </button>
                        </li>
                    </ul>
                );
            }
            const spinnerElm = document.getElementById('orderLoadingSpinner');
            if (spinnerElm) {
                spinnerElm.className = "d-none";
            }

            setOrderList(orderElmList);
        }).catch((err: any)=>{
            if ('Invalid Session'===err) {
                navigate('/signon/')
            } else {
                console.log(`Failed getting order list: ${JSON.stringify(err)}`);
                throw err;
            }
        });
    }, [])

    return (
        <div>
            <NavBar/>
            <button type="button"
                    className="btn btn-outline-info add-order-btn"
                    onClick={addNewOrder}>
                +
            </button>
            <div className="col-xs-1 d-flex justify-content-center">
                <div className="card">
                    <div className="card-body">
                        <h5 className="card-title" id="orderCardTitle">Orders</h5>
                        {orderList}
                        <div className="spinner-border" role="status" id="orderLoadingSpinner">
                            <span className="sr-only">Loading...</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
