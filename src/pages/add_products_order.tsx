import React from "react"
import {orderDb, NewOrder, DeliverableOrderIf} from "../js/ordersdb"
import { navigate } from "gatsby"
import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"


export default (params: any) => {
    if (undefined === params.location.state) { return(<div/>); } //only used for building 
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
        navigate('/order_step_1/');
    }

    const onFormSubmission = (event: any) => {
        event.preventDefault();
        event.stopPropagation();

        let totalDue = currency(0.0);
        const items: Map<string, number> = new Map<string, number>();
        for (let [productId, product] of fundraiserConfig.products()) {
            const formId = `form${productId}`;
            const numOrdered = parseInt((document.getElementById(formId) as HTMLInputElement).value);
            if (0 < numOrdered) {
                items.set(productId, numOrdered);
                totalDue = totalDue.add((product as any).cost.multiply(numOrdered));
            }
        }
        let mulchOrder = {
            totalDue: totalDue,
            kind: fundraiserConfig.kind(),
            items: items
        };
        currentOrder.deliverables.set(deliveryDate, (mulchOrder as DeliverableOrderIf));

        navigate('/order_step_1/');
    }

    const products=[];
    for (let [productId, product] of fundraiserConfig.products()) {
        const formId = `form${productId}`;
        let numOrdered = undefined;
        if (undefined !== deliveryDateOrder) {
            if (deliveryDateOrder.items) {
                numOrdered = deliveryDateOrder.items.get(productId);
            }
        }
        products.push(
            <div className="form-group row col-sm-12" key={`${formId}RowId`}>
                <label htmlFor={formId}>{product.costDescription}: {USD(product.cost).format()}</label>
                <input type="number" className="form-control" id={formId}
                       defaultValue={numOrdered}
                       placeholder={product.label}/>
            </div>
        );

    };

    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <div className="card">
                <div className="card-body">
                    <h5 className="card-title">Add {fundraiserConfig.description()} Order for {deliveryDate}</h5>
                    <form onSubmit={onFormSubmission}>

                        {products}

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
