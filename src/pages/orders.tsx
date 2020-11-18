import React from "react"
import {ListGroup, Card} from "react-bootstrap"
import NavBar from "../components/navbar"
import { navigate } from "gatsby"
import {orderDb, OrderIf} from "../js/ordersdb"

export default function orders() {

    const addNewOrder=()=>{
        console.log("Add new order");
        navigate('/order_step_1', {replace: true});
    };
    const editOrder=(order: OrderIf)=>{
        console.log(`Editorder ${order.id}`);
    };

    const orders = [];
    for (const order of orderDb.orders()) {
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
            <div className="col-xs-1 d-flex justify-content-center">
                <Card>
                    <Card.Body>
                        <Card.Title>Orders</Card.Title>
                        {orders}
                    </Card.Body>
                </Card>
            </div>
        </div>
    );
}
