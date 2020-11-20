import React, { useState } from "react"
import {Card, Form, Button, Col} from "react-bootstrap"
import {orderDb, NewOrder, DeliverableOrderIf} from "../js/ordersdb"
import { navigate } from "gatsby"
import currency from "currency.js"


export default function addDonation() {
    const [validated, setValidated] = useState(false);

    let currentOrder: NewOrder = orderDb.getCurrentOrder();

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

        const donationOrder: DeliverableOrderIf = {
            totalDue: currency((document.getElementById('formDonationAmount') as HTMLInputElement).value),
            kind: 'donation'
        };

        currentOrder.deliverables.set('donation', donationOrder);

        navigate('/order_step_1', {replace: true});
    }

    let donationAmt = currency(0.0);
    let currentDonation = currentOrder.deliverables.get('donation');
    if (undefined!==currentDonation) {
        donationAmt=currentDonation.totalDue;
    }

    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <Card>
                <Card.Body>
                    <Card.Title>Add Donation</Card.Title>
                    <Form noValidate validated={validated} onSubmit={onFormSubmission}>
                        <Form.Row>
                            <Form.Group as={Col} md="12" controlId="formDonationAmount">
                                <Form.Control required type="number"
                                              placeholder="Enter Donation Amount"
                                              defaultValue={donationAmt.toString()}
                                              onInput={doesSubmitGetEnabled} />
                            </Form.Group>
                        </Form.Row>
                        <Button variant="primary" className="my-2" type="submit" onClick={onCancelItem}>
                            Back
                        </Button>
                        <Button variant="primary" className="my-2 float-right" type="submit"
                                disabled={0.0===donationAmt.value} id="formDonationSubmit">
                            Add
                        </Button>
                    </Form>
                </Card.Body>
            </Card>
        </div>
    );
    
}
