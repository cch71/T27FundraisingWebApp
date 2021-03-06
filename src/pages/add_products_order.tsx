import React, { useEffect } from "react"
import {orderDb, Order} from "../js/ordersdb"
import { navigate } from "gatsby"
import currency from "currency.js"
import {FundraiserConfig, getFundraiserConfig} from "../js/fundraiser_config"
import {onNonNumsKeyPress} from "../js/utils"

const USD = (value: currency) => currency(value, { symbol: "$", precision: 2 });

export default (params: any) => {
    if (undefined === params.location.state) { return(<div/>); } //only used for building

    // Could be undefined if this is a new order
    const currentDeliveryId = params?.location?.state?.deliveryId;
    const isAdmin = params?.location?.state?.isAdmin;
    const isOrderReadOnly = params?.location?.state?.isOrderReadOnly;
    console.log(`Current Delivery ID: ${currentDeliveryId}`);

    const currentOrder: Order = orderDb.getActiveOrder();
    if (!currentOrder) {
        navigate('/');
        return(<div/>);
    }

    const deliveryDateOrder = currentOrder.deliveryId;
    const fundraiserConfig: FundraiserConfig = getFundraiserConfig();
    if (!fundraiserConfig) {
        alert("Failed to load fundraiser config");
        return(<div/>);
    }

    const deliveryDateOpts = [];
	const deliveryDates = (isAdmin || isOrderReadOnly) ?
						fundraiserConfig.deliveryDates() :
						fundraiserConfig.validDeliveryDates();
    for (const [frDeliveryId, frDeliveryLabel] of deliveryDates) {
        if ('donation'===frDeliveryId) { continue; }
        deliveryDateOpts.push(
            <option value={frDeliveryId} key={frDeliveryId}>{frDeliveryLabel}</option>
        );
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

        const items: Record<string, number> = {};
        for (const product of fundraiserConfig.products()) {
            const formId = `form${product.id}`;
            const numOrdered = parseInt((document.getElementById(formId) as HTMLInputElement).value);
            if (0 < numOrdered) {
                items[product.id] = numOrdered;
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

        if ('0.00' === totalDue.toString()) {
            delete currentOrder.deliveryId;
            delete currentOrder.products;
            delete currentOrder.productsCost;
        } else {
            console.log(`Setting Order: ${selectedDeliveryId}`);
            currentOrder['deliveryId'] = selectedDeliveryId;
            currentOrder['products'] = items;
            currentOrder['productsCost'] = totalDue;
        }
        navigate('/order_step_1/');
    }

	// Populate form elements
    const productElms=[];
    for (const prod of fundraiserConfig.products()) {
        console.log(`Handling Product: ${JSON.stringify(prod)}`);
        const formId = `form${prod.id}`;
        let numOrdered = undefined;
        if (undefined !== deliveryDateOrder && currentOrder.products) {
            numOrdered = currentOrder.products[prod.id];
        }

        let productLabel = `${prod.label} Price: ${USD(prod.unitPrice).format()}`;
        for (const priceBreak of prod.priceBreaks) {
            const unitsNeeded = priceBreak.gt;
            const unitPrice = USD(currency(priceBreak.unitPrice)).format();
            productLabel += ` [>${unitsNeeded}=${unitPrice}] `;
        }

        productElms.push(
            <div className="row mb-2 col-sm-12" key={`${formId}RowId`}>
                <label htmlFor={formId}>{productLabel}</label>
                <input type="number" min="0" className="form-control" id={formId}
                       onKeyPress={onNonNumsKeyPress}
                       defaultValue={numOrdered} autoComplete="off"
                       placeholder={0}/>
            </div>
        );

    };

	useEffect(() => {
		if (!isAdmin && isOrderReadOnly) {
			jQuery('#productForm :input').attr('readonly','readonly');
			jQuery('#formSelectDeliveryDate').attr('disabled','disabled');
		}
	}, [])


	return (
		<div className="col-xs-1 d-flex justify-content-center">
			<div className="card">
				<div className="card-body">
					<h5 className="card-title">{fundraiserConfig.description()} Order</h5>
					<form id="productForm" onSubmit={onFormSubmission}>

						<div className="row mb-3 col-sm-12">
							<label htmlFor="formSelectDeliveryDate">Select Delivery Date</label>
							<select className="custom-select" id="formSelectDeliveryDate" defaultValue={currentDeliveryId}>
								{deliveryDateOpts}
							</select>
						</div>

						{productElms}

						<button type="button" className="btn btn-primary my-2" onClick={onCancelItem}>
							Cancel
						</button>
						<button type="submit" className="btn btn-primary my-2 float-end"
							style={{display: ((isAdmin || !isOrderReadOnly)?"block":"none")}}
						>
							Submit
						</button>
					</form>
				</div>
			</div>
		</div>
	);
}
