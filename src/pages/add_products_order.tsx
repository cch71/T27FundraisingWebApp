import React, { useState } from "react"
import {Card, Form, Button, Col} from "react-bootstrap"
import {orderDb, NewOrder, DeliverableOrderIf} from "../js/ordersdb"
import { navigate } from "gatsby"
import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"


export default (params: any) => {
    const [validated, setValidated] = useState(false);

    const deliveryDate = params.location.state.deliveryDate;
    const currentOrder: NewOrder = orderDb.getCurrentOrder();
    const deliveryDateOrder = currentOrder.deliverables.get(deliveryDate);
    const fundraiserConfig: FundraiserConfig = getFundraiserConfig();
    if (!fundraiserConfig) {
        alert("Failed to load fundraiser config");
        return(<div/>);
    }

    const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });

    const doesSubmitGetEnabled = (event: any)=>{
        if (event.currentTarget.value) {
            (document.getElementById('formAddProductsSubmit') as HTMLButtonElement).disabled = false;
        } else {
            (document.getElementById('formAddProductsSubmit') as HTMLButtonElement).disabled = true;
        }
    };

    const onCancelItem = ()=>{
        navigate('/order_step_1/', {replace: true});
    }

    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();

        let totalDue = currency(0.0);
        const items: Map<string, number> = new Map<string, number>();
        for (let [productId, product] of Object.entries(fundraiserConfig.products)) {
            const formId = `form${productId}`;
            const numOrdered = parseInt((document.getElementById(formId) as HTMLInputElement).value);
            if (0 < numOrdered) {
                items.set(productId, numOrdered);
                totalDue = totalDue.add((product as any).cost.multiply(numOrdered));
            }
        }
        let mulchOrder = {
            totalDue: totalDue,
            kind: fundraiserConfig.kind,
            items: items
        };
        currentOrder.deliverables.set(deliveryDate, (mulchOrder as DeliverableOrderIf));

        navigate('/order_step_1/', {replace: true});
    }

    const products=[];
    for (let entry of Object.entries(fundraiserConfig.products)) {
        let productId: string = entry[0];
        let product: any = entry[1];
        const formId = `form${productId}`;
        let numOrdered = undefined;
        if (undefined !== deliveryDateOrder) {
            if (deliveryDateOrder.items) {
                numOrdered = deliveryDateOrder.items.get(productId);
            }
        }
        products.push(
            <Form.Row key={`${formId}RowId`}>
                <Form.Group as={Col} md="12" controlId={formId} >
                    <Form.Label>{product.costDescription}: {USD(product.cost).format()}</Form.Label>
                    <Form.Control required type="number"
                                  placeholder={product.label}
                                  defaultValue={numOrdered} />
                </Form.Group>
            </Form.Row>
        );

    };

    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <Card>
                <Card.Body>
                    <Card.Title>Add {fundraiserConfig.description} Order for {deliveryDate}</Card.Title>
                    <Form noValidate validated={validated} onSubmit={onFormSubmission}>
                        {products}
                        <Button variant="primary" className="my-2" onClick={onCancelItem}>
                            Back
                        </Button>
                        <Button variant="primary" className="my-2 float-right" type="submit"
                                id="formAddProductsSubmit">
                            Add
                        </Button>
                    </Form>
                </Card.Body>
            </Card>
        </div>
    );
    
}
