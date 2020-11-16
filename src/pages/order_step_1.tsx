import React, { useState } from "react"
import NavBar from "../components/navbar"
import {Card, Form, Button, ButtonGroup, Col} from "react-bootstrap"



export default function orderStep1() {
    const [validated, setValidated] = useState(false);

    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();

        const form = event.currentTarget;

        //const loginId = form[0].value;
        //const pw = form[1].value;
        //console.log(`Form submttted uid: pw: ${loginId} ${pw}`)
    }

    const onAddOrder = (event: any)=>{
        const form = event.currentTarget.form;
        event.preventDefault();
        event.stopPropagation();
        console.log(`Adding New Fundraising Order`);
    };

    const onAddDonation = (event: any)=>{
        const form = event.currentTarget.form;
        event.preventDefault();
        event.stopPropagation();
        console.log(`Adding New Donation`);
    };

    
    /* <Form.Text className="text-muted">
       If you don't know what yours is check with TODO: Email
       </Form.Text>
     */
    return (
        <div>
            <NavBar/>
            <div className="col-xs-1 d-flex justify-content-center">
                <Card>
                    <Card.Body>
                        <Card.Title>Customer Information</Card.Title>
                        <Form noValidate validated={validated} onSubmit={onFormSubmission}>
                            <Form.Row>
                                <Form.Group as={Col} md="12" controlId="formCustomerName">
                                    <Form.Label>Customer Name</Form.Label>
                                    <Form.Control required type="text" placeholder="Enter Customer Name" />
                                </Form.Group>
                            </Form.Row>
                            <Form.Row>
                                <Form.Group as={Col} md="6" controlId="formAddr1">
                                    <Form.Label>Address 1</Form.Label>
                                    <Form.Control required type="text" placeholder="Address 1" />
                                </Form.Group>

                                <Form.Group as={Col} md="6" controlId="formAddr2">
                                    <Form.Label>Address 2</Form.Label>
                                    <Form.Control type="text" placeholder="Address 2" />
                                </Form.Group>
                            </Form.Row>

                            <Form.Row>
                                <Form.Group as={Col} md="7" controlId="formCity">
                                    <Form.Label>City</Form.Label>
                                    <Form.Control required type="text" placeholder="City" />
                                </Form.Group>

                                <Form.Group as={Col} md="2" controlId="formState">
                                    <Form.Label>State</Form.Label>
                                    <Form.Control type="text" placeholder="State" defaultValue="TX" />
                                </Form.Group>

                                <Form.Group as={Col} md="3" controlId="formZip">
                                    <Form.Label>Zip</Form.Label>
                                    <Form.Control type="text" placeholder="Zip" />
                                </Form.Group>
                            </Form.Row>

                            <Form.Row>
                                <Form.Group as={Col} md="12" controlId="formSpecialInstructions">
                                    <Form.Label>Special Instructions</Form.Label>
                                    <Form.Control as="textarea" rows={4} />
                                </Form.Group>
                            </Form.Row>

                            <ButtonGroup aria-label="Basic example">
                                <Button variant="primary" className="mr-2" type="button" onClick={onAddDonation}>
                                    Add Donation
                                </Button>

                                <Button variant="primary" type="button" onClick={onAddOrder}>
                                    Add Order
                                </Button>

                                <Button variant="primary" className="ml-2" type="submit">
                                    Submit
                                </Button>
                            </ButtonGroup>
                        </Form>
                    </Card.Body>
                </Card>
            </div>
        </div>
    );


}
