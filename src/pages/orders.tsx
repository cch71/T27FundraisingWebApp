import React from "react"
import {ListGroup} from "react-bootstrap"
import NavBar from "../components/navbar"
import {orderDb, OrderIf} from "../js/ordersdb"

export default function orders() {

    const addNewOrder=()=>{
        console.log("Add new order");
    };
    const editOrder=(order: OrderIf)=>{
        console.log(`Editorder ${order.id}`);
    };

    const orders = [];
    for (const order of orderDb.getOrders()) {
        orders.push(
            <ListGroup horizontal='lg' onClick={()=>{editOrder(order)}} key={order.id} className='my-2'>
                <ListGroup.Item>{order.name}</ListGroup.Item>
                <ListGroup.Item>{order.addr1} {order.addr2} {order.city} {order.state} {order.zip}</ListGroup.Item>
                <ListGroup.Item>{order.totalDue}</ListGroup.Item>
            </ListGroup>);
    }

    
    return (
        <div>
            <NavBar/>
            <button type="button"
                    className="btn btn-outline-light add-order-btn"
                    onClick={addNewOrder}>
                +
            </button>
            
            <div>Orders</div>
            {orders}
        </div>
    );
}

