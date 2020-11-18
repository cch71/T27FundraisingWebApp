import React, { useState } from "react"
import {Card, Form, Button, Col} from "react-bootstrap"
import {orderDb, NewOrder, OrderItemIf} from "../js/ordersdb"
import { navigate } from "gatsby"


export default function addMulch(params: any) {
    const [validated, setValidated] = useState(false);

    const deliveryDate = params.location.state.deliveryDate;
    const currentOrder: NewOrder = orderDb.getCurrentOrder();
    const fundraisingConfig: any = orderDb.getCurrentFundraiserConfig();
    const pricePerBag = fundraisingConfig.pricePerBag;
    const pricePerSpread = fundraisingConfig.pricePerSpread;

    const doesSubmitGetEnabled = (event: any)=>{
        if (event.currentTarget.value) {
            (document.getElementById('formDonationSubmit') as HTMLButtonElement).disabled = false;
        } else {
            (document.getElementById('formDonationSubmit') as HTMLButtonElement).disabled = true;
        }
    };

    const onCancelItem = ()=>{
        navigate('/order_step_1', {replace: true});
    }

    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();

        const bags = parseInt((document.getElementById('formBags') as HTMLInputElement).value);
        const toSpread = parseInt((document.getElementById('formNumToSpread') as HTMLInputElement).value);
        const mulchOrder = {
            totalBags: bags,
            totalSpread: toSpread,
            totalDue: (bags * pricePerBag) + (toSpread * pricePerSpread),
            kind: 'mulch'
        };

        currentOrder.orderItems.set(deliveryDate, (mulchOrder as OrderItemIf));

        navigate('/order_step_1', {replace: true});
    }

    //let totalDue = 0.0;
    let orderItem = currentOrder.orderItems.get(deliveryDate);
    let totalBags = 0;
    let totalSpread = 0;
    if (undefined!==orderItem) {
        //totalDue=currentOrder.totalDue;
        totalBags=(orderItem as any).totalBags;
        totalSpread=(orderItem as any).totalSpread;
    }

    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <Card>
                <Card.Body>
                    <Card.Title>Add Mulch Order for {deliveryDate}</Card.Title>
                    <div>Cost per bag: ${pricePerBag}</div>
                    <div>Cost per bag to spread: ${pricePerSpread}</div>
                    <Form noValidate validated={validated} onSubmit={onFormSubmission}>
                        <Form.Row>
                            <Form.Group as={Col} md="12" controlId="formBags">
                                <Form.Label>Number of Bags</Form.Label>
                                <Form.Control required type="number"
                                              placeholder="Bags"
                                              defaultValue={totalBags} />
                            </Form.Group>
                        </Form.Row>
                        <Form.Row>
                            <Form.Group as={Col} md="12" controlId="formNumToSpread">
                                <Form.Label>Bags to Spread</Form.Label>
                                <Form.Control required type="number"
                                              placeholder="Bags to spread"
                                              defaultValue={totalSpread} />
                            </Form.Group>
                        </Form.Row>
                        <Button variant="primary" className="my-2" type="submit" onClick={onCancelItem}>
                            Back
                        </Button>
                        <Button variant="primary" className="my-2 float-right" type="submit"
                                id="formAddMulchSubmit">
                            Add
                        </Button>
                    </Form>
                </Card.Body>
            </Card>
        </div>
    );
    
}
