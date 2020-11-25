import React from "react"
import {orderDb, Order, OrdersForDeliveryDate} from "../js/ordersdb"
import { navigate } from "gatsby"
import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"


export default (params: any) => {
    if (undefined === params.location.state) { return(<div/>); } //only used for building 
    const deliveryDate = params?.location?.state?.deliveryLabel;
    const deliveryId = params?.location?.state?.deliveryId;
    const currentOrder: Order = orderDb.getActiveOrder();
    if (!currentOrder) {
        navigate('/');
        return(<div/>);
    }
    const deliveryDateOrder = currentOrder.orderByDelivery.get(deliveryId);
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
        navigate('/order_step_1/');
    }

    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();
        
        let totalDue = currency(0.0);
        const items: Map<string, number> = new Map<string, number>();
        for (const product of fundraiserConfig.products()) {
            const formId = `form${product.id}`;
            const numOrdered = parseInt((document.getElementById(formId) as HTMLInputElement).value);
            if (0 < numOrdered) {
                items.set(product.id, numOrdered);
                totalDue = totalDue.add((product as any).cost.multiply(numOrdered));
            }
        }
        let mulchOrder = {
            amountDue: totalDue,
            kind: fundraiserConfig.kind(),
            items: items
        };
        console.log(`Setting Order: ${deliveryId}`);
        currentOrder.orderByDelivery.set(deliveryId, (mulchOrder as OrdersForDeliveryDate));
        
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
        productElms.push(
            <div className="form-group row col-sm-12" key={`${formId}RowId`}>
                <label htmlFor={formId}>{prod.costDescription}: {USD(prod.cost).format()}</label>
                <input type="number" className="form-control" id={formId}
                       defaultValue={numOrdered}
                       placeholder={prod.label}/>
            </div>
        );

    };

    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <div className="card">
                <div className="card-body">
                    <h5 className="card-title">Add {fundraiserConfig.description()} Order for {deliveryDate}</h5>
                    <form onSubmit={onFormSubmission}>

                        {productElms}

                        <button type="button" className="btn btn-primary my-2" onClick={onCancelItem}>
                            Back
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
