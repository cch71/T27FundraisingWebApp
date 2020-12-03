import React from "react"
import {orderDb, Order, OrdersForDeliveryDate} from "../js/ordersdb"
import { navigate } from "gatsby"
import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"

const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });

export default (params: any) => {
    if (undefined === params.location.state) { return(<div/>); } //only used for building

    // Could be undefined if this is a new order
    const currentDeliveryId = params?.location?.state?.deliveryId;
    console.log(`Current Delivery ID: ${currentDeliveryId}`);
    
    const currentOrder: Order = orderDb.getActiveOrder();
    if (!currentOrder) {
        navigate('/');
        return(<div/>);
    }
    
    const deliveryDateOrder = currentOrder.orderByDelivery.get(currentDeliveryId);
    const fundraiserConfig: FundraiserConfig = getFundraiserConfig();
    if (!fundraiserConfig) {
        alert("Failed to load fundraiser config");
        return(<div/>);
    }

    // TODO: probably better way to do this
    const deliveryDateOpts = []
    for (const [frDeliveryId, frDeliveryLabel] of fundraiserConfig.validDeliveryDates()) {
        if ('donation'===frDeliveryId) { continue; }
        if (frDeliveryId === currentDeliveryId || !currentOrder.orderByDelivery.has(frDeliveryId)) {
            deliveryDateOpts.push(
                <option value={frDeliveryId} key={frDeliveryId}>{frDeliveryLabel}</option>
            );
        }
    }


            // Controls when the submit button gets enabled
    const doesSubmitGetEnabled = (event: any)=>{
        if (event.currentTarget.value) {
            (document.getElementById('formAddProductsSubmit') as HTMLButtonElement).disabled = false;
        } else {
            (document.getElementById('formAddProductsSubmit') as HTMLButtonElement).disabled = true;
        }
    };

    // Called to cancel this operation and go back to order screen
    const onCancelItem = ()=>{
        navigate('/order_step_1/');
    }

    // Called when this gets added to the order
    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();
        
        let totalDue = currency(0.0);
        const selectedDeliveryId = (document.getElementById('formSelectDeliveryDate') as HTMLSelectElement).value;
        console.log(`Saving For Delivery ID: ${selectedDeliveryId}`);
        if (currentDeliveryId && currentDeliveryId!==selectedDeliveryId) {
            currentOrder.orderByDelivery.delete(currentDeliveryId);
        }

        const items: Map<string, number> = new Map<string, number>();
        for (const product of fundraiserConfig.products()) {
            const formId = `form${product.id}`;
            const numOrdered = parseInt((document.getElementById(formId) as HTMLInputElement).value);
            if (0 < numOrdered) {
                items.set(product.id, numOrdered);
                let rate = currency(product.unitPrice);
                // Handle Price product price breaks if any
                for (const priceBreak of product.priceBreaks) {
                    const unitsNeeded = priceBreak.gt;
                    if (numOrdered > unitsNeeded) {
                        rate = currency(priceBreak.unitPrice);
                        console.log(`Ordered: ${numOrdered}  UN: ${unitsNeeded} Rate: ${rate}`);
                    }
                }
                totalDue = totalDue.add(rate.multiply(numOrdered));
            }
        }

        if (0.0===totalDue.value) {
            currentOrder.orderByDelivery.delete(selectedDeliveryId);
        } else {
        
            let productOrder = {
                amountDue: totalDue,
                kind: fundraiserConfig.kind(),
                items: items
            };
            console.log(`Setting Order: ${selectedDeliveryId}`);
            currentOrder.orderByDelivery.set(selectedDeliveryId, (productOrder as OrdersForDeliveryDate));
        }
        navigate('/order_step_1/');
    }

    const productElms=[];
    for (const prod of fundraiserConfig.products()) {
        console.log(`Handling Product: ${JSON.stringify(prod)}`);
        const formId = `form${prod.id}`;
        let numOrdered = undefined;
        if (undefined !== deliveryDateOrder) {
            if (deliveryDateOrder.items) {
                numOrdered = deliveryDateOrder.items.get(prod.id);
            }
        }

        let productLabel = `${prod.label} Price: ${USD(prod.unitPrice).format()}`;
        for (const priceBreak of prod.priceBreaks) {
            const unitsNeeded = priceBreak.gt;
            const unitPrice = USD(currency(priceBreak.unitPrice)).format();
            productLabel += ` [>${unitsNeeded}=${unitPrice}] `;
        }
        
        productElms.push(
            <div className="form-group row col-sm-12" key={`${formId}RowId`}>
                <label htmlFor={formId}>{productLabel}</label>
                <input type="number" className="form-control" id={formId}
                       defaultValue={numOrdered}
                       placeholder={0}/>
            </div>
        );
        
    };

    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <div className="card">
                <div className="card-body">
                    <h5 className="card-title">Add {fundraiserConfig.description()} Order</h5>
                    <form onSubmit={onFormSubmission}>
                        
                        <div className="form-group row col-sm-12">
                            <label htmlFor="formSelectDeliveryDate">Select Delivery Date</label>
                            <select className="custom-select" id="formSelectDeliveryDate" defaultValue={currentDeliveryId}>
                                {deliveryDateOpts}
                            </select>
                        </div>
                            
                        {productElms}
                        
                        <button type="button" className="btn btn-primary my-2" onClick={onCancelItem}>
                            Cancel
                        </button>
                        <button type="submit" className="btn btn-primary my-2 float-right" id="formAddProductsSubmit">
                            Add                            
                        </button>
                        
                    </form>
                </div>
            </div>
        </div>
    );
}
