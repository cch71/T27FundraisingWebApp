import React, { useState } from "react"
import NavBar from "../components/navbar"
import {Card, Form, Button, ListGroup, Col} from "react-bootstrap"
import {orderDb, NewOrder} from "../js/ordersdb"
import OrderItem from "../components/order_item" //TODO: Rename DeliveryOrderSummary
import { navigate } from "gatsby"
import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"


export default (location: any)=>{
    const [validated, setValidated] = useState(false);
    const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });

    const fundraiserConfig: FundraiserConfig = getFundraiserConfig();
    if (!fundraiserConfig) {
        alert("Failed to load fundraiser config");
        return(<div/>);
    }


    let currentOrder: NewOrder = orderDb.getCurrentOrder();

    const saveCurrentOrder = ()=>{
        currentOrder.firstName = (document.getElementById('formFirstName') as HTMLInputElement).value;
        currentOrder.lastName = (document.getElementById('formLastName') as HTMLInputElement).value;
        currentOrder.email = (document.getElementById('formEmail') as HTMLInputElement).value;
        currentOrder.phone = (document.getElementById('formPhone') as HTMLInputElement).value;
        currentOrder.addr1 = (document.getElementById('formAddr1') as HTMLInputElement).value;
        currentOrder.addr2 = (document.getElementById('formAddr2') as HTMLInputElement).value;
        currentOrder.city = (document.getElementById('formCity') as HTMLInputElement).value;
        currentOrder.state = (document.getElementById('formState') as HTMLInputElement).value;
        currentOrder.zip = (document.getElementById('formZip') as HTMLInputElement).value;
        currentOrder.specialInstructions =
            (document.getElementById('formSpecialInstructions') as HTMLInputElement).value;
        currentOrder.cashPaid =
            currency((document.getElementById('formCashAmount') as HTMLInputElement).value);
        currentOrder.checkPaid =
            currency((document.getElementById('formCheckAmount') as HTMLInputElement).value);
        console.log(`Current Order ${JSON.stringify(currentOrder, null, 2)}`);
        orderDb.setCurrentOrder(currentOrder);
    }

    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();

    }

    const onAddOrder = (event: any)=>{
        event.preventDefault();
        event.stopPropagation();

        saveCurrentOrder()

        const btn = event.currentTarget;
        console.log(`Add New Fundraising Order for ${btn.dataset.deliverydate}`);

        navigate('/add_products_order/', {replace: true, state: {deliveryDate: btn.dataset.deliverydate}});
    };

    const onAddDonation = (event: any)=>{
        event.preventDefault();
        event.stopPropagation();
        console.log(`Adding New Donation`);

        saveCurrentOrder()
        navigate('/add_donations/', {replace: true});
    };

    const doesSubmitGetEnabled = (event: any)=>{
        if (event.currentTarget.value) {
            (document.getElementById('formOrderSubmit') as HTMLButtonElement).disabled = false;
        } else {
            (document.getElementById('formOrderSubmit') as HTMLButtonElement).disabled = true;
        }
    };

    let totalDue = currency(0.0);
    const recalculateTotal = ()=> {
        totalDue = currency(0.0);
        for (let deliverable of currentOrder.deliverables.values()) {
            console.log(`Found Order: ${deliverable.totalDue}`);
            totalDue = totalDue.add(deliverable.totalDue);
        }
        const totElm = document.getElementById('orderTotalDue');
        if (null!==totElm) {
            totElm.innerText = `Total Due: ${USD(totalDue).format()}`;
        }
    }

    const ordersByDeliveryBtns = []
    for (const deliveryDate of fundraiserConfig.validDeliveryDates()) {
        const onClickHandler = ("donation" === deliveryDate)? onAddDonation : onAddOrder;

        ordersByDeliveryBtns.push(
            <ListGroup.Item key={deliveryDate}>
                <OrderItem onClick={onClickHandler} deliveryDate={deliveryDate} onDelete={recalculateTotal} />
            </ListGroup.Item>
        );
    }

    const hoods=[];
    for (let hood of fundraiserConfig.neighborhoods()) {
        hoods.push(<option key={hood}>{hood}</option>);
    }

    const stateAbbreviations = [
        'AL','AK','AS','AZ','AR','CA','CO','CT','DE','DC','FM','FL','GA',
        'GU','HI','ID','IL','IN','IA','KS','KY','LA','ME','MH','MD','MA',
        'MI','MN','MS','MO','MT','NE','NV','NH','NJ','NM','NY','NC','ND',
        'MP','OH','OK','OR','PW','PA','PR','RI','SC','SD','TN','TX','UT',
        'VT','VI','VA','WA','WV','WI','WY'
    ];

    const states=[];
    for (let st of stateAbbreviations) {
        states.push(<option key={st}>{st}</option>);
    }

    recalculateTotal();


    return (
        <div>
            <NavBar/>
            <div className="col-xs-1 d-flex justify-content-center">
                <Card>
                    <Card.Body>
                        <Card.Title>Customer Information</Card.Title>
                        <Form noValidate validated={validated} onSubmit={onFormSubmission}>
                            <Form.Row>
                                <Form.Group as={Col} md="6" controlId="formFirstName">
                                    <Form.Label>First Name</Form.Label>
                                    <Form.Control required type="text" placeholder="First Name"
                                                  onInput={doesSubmitGetEnabled}
                                                  defaultValue={currentOrder.firstName}/>
                                    <Form.Text className="text-muted">* Required</Form.Text>
                                </Form.Group>
                                <Form.Group as={Col} md="6" controlId="formLastName">
                                    <Form.Label>Last Name</Form.Label>
                                    <Form.Control required type="text" placeholder="Last Name"
                                                  onInput={doesSubmitGetEnabled}
                                                  defaultValue={currentOrder.lastName}/>
                                    <Form.Text className="text-muted">* Required</Form.Text>
                                </Form.Group>
                            </Form.Row>
                            <Form.Row>
                                <Form.Group as={Col} md="4" controlId="formNeighborhood">
                                    <Form.Label>Neighborhood</Form.Label>
                                    <Form.Control as="select">
                                        {hoods}
                                    </Form.Control>
                                </Form.Group>
                                <Form.Group as={Col} md="4" controlId="formPhone">
                                    <Form.Label>Phone</Form.Label>
                                    <Form.Control type="text" placeholder="Phone"
                                                  defaultValue={currentOrder.phone} />
                                </Form.Group>
                                <Form.Group as={Col} md="4" controlId="formEmail">
                                    <Form.Label>Email</Form.Label>
                                    <Form.Control type="text" placeholder="Email"
                                                  defaultValue={currentOrder.email} />
                                </Form.Group>
                            </Form.Row>
                            <Form.Row>
                                <Form.Group as={Col} md="6" controlId="formAddr1">
                                    <Form.Label>Address 1</Form.Label>
                                    <Form.Control required type="text" placeholder="Address 1" defaultValue={currentOrder.addr1}/>
                                </Form.Group>

                                <Form.Group as={Col} md="6" controlId="formAddr2">
                                    <Form.Label>Address 2</Form.Label>
                                    <Form.Control type="text" placeholder="Address 2" defaultValue={currentOrder.addr2} />
                                </Form.Group>
                            </Form.Row>
                            <Form.Row>
                                <Form.Group as={Col} md="7" controlId="formCity">
                                    <Form.Label>City</Form.Label>
                                    <Form.Control required type="text" placeholder="City" defaultValue={currentOrder.city} />
                                </Form.Group>

                                <Form.Group as={Col} md="2" controlId="formState">
                                    <Form.Label>State</Form.Label>
                                    <Form.Control as="select" defaultValue='TX'>
                                        {states}
                                    </Form.Control>
                                </Form.Group>

                                <Form.Group as={Col} md="3" controlId="formZip">
                                    <Form.Label>Zip</Form.Label>
                                    <Form.Control type="text" placeholder="Zip" defaultValue={currentOrder.zip} />
                                </Form.Group>
                            </Form.Row>

                            <Form.Row>
                                <Form.Group as={Col} md="12" controlId="formSpecialInstructions">
                                    <Form.Label>Special Instructions</Form.Label>
                                    <Form.Control as="textarea" rows={4} defaultValue={currentOrder.specialInstructions} />
                                </Form.Group>
                            </Form.Row>

                            <ListGroup>
                                {ordersByDeliveryBtns}
                            </ListGroup>

                            <Form.Row>
                                <Form.Group as={Col} md="6" controlId="formCashAmount" >
                                    <Form.Label>Amount paid with cash</Form.Label>
                                    <Form.Control required type="number"
                                                  placeholder="Amount paid with cash"
                                                  defaultValue={currentOrder.cashPaid.toString()} />
                                </Form.Group>
                                <Form.Group as={Col} md="6" controlId="formCheckAmount" >
                                    <Form.Label>Amount paid with check</Form.Label>
                                    <Form.Control required type="number"
                                                  placeholder="Amount paid with check"
                                                  defaultValue={currentOrder.checkPaid.toString()}  />
                                </Form.Group>
                            </Form.Row>


                            <div>Total Paid: $Calculation TBD</div>
                            <div id="orderTotalDue">Total Due: {USD(totalDue).format()}</div>


                            <Button variant="primary" className="my-2 float-right" type="submit" disabled id="formOrderSubmit">
                                Submit
                            </Button>
                        </Form>
                    </Card.Body>
                </Card>
            </div>
        </div>
    );


}
