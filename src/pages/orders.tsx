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
        navigate('/order_step_1/', {replace: true});
    };
    const editOrder=(order: OrderIf)=>{
        console.log(`Editorder ${order.orderId}`);
    };

    const orders = [];
    for (const order of orderDb.orders()) {
        orders.push(
            <ListGroup horizontal='lg' onClick={()=>{editOrder(order)}} key={order.orderId} className='my-2'>
                <ListGroup.Item style={{width: '150px'}}>{order.firstName}, {order.lastName}</ListGroup.Item>
                <ListGroup.Item style={{width: '400px'}}>{order.addr1} {order.addr2} {order.city} {order.state} {order.zip}</ListGroup.Item>
                <ListGroup.Item style={{width: '150px'}}>{USD(order.totalDue).format()}</ListGroup.Item>
                <ListGroup.Item>
                    <Button variant="outline-danger" className="mx-1 float-right">X</Button>
                    <Button variant="outline-info" className="float-right">I</Button>
                </ListGroup.Item>
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
